# syntax=docker/dockerfile:1
#
# Imagem do openMTR: builder Rust 1.100 -> runtime Debian bookworm-slim.
# O runtime entrega a CLI `openmtr`; as bibliotecas (napi/UniFFI) são apenas
# inspecionadas no builder e relatadas, não copiadas para a imagem final.

#_builder
FROM rust:1.100-slim-bookworm AS builder

# O workspace usa reqwest/rustls-tls (ver core/Cargo.toml: `native-http`).
# Não há openssl/native-tls no Cargo.lock: libssl-dev é desnecessário.
# build-essential: toolchain C para `cc`/`ring` (TLS) e confirmados os scripts de build.
# perl: `ring` 0.17 precisa dele quando .git está presente (COPY . . traz .git
# somente se .dockerignore permitir; manter garante o build em ambos os casos).
# O build de napi NÃO roda aqui: napi-build apenas emite flags de linker, e o
# pacote Node (@napi-rs/cli) não é necessário para `cargo build`.
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential perl \
    && rm -rf /var/lib/apt/lists/*

# ARG explícito: por padrão NÃO instalamos o uniffi-bindgen Python (rede/pip).
# Habilite com `--build-arg ENABLE_PYTHON_BINDINGS=1`; a versão segue o crate
# uniffi do Cargo.lock (0.28.x). O comando não engole erros.
ARG ENABLE_PYTHON_BINDINGS=0
RUN if [ "$ENABLE_PYTHON_BINDINGS" = "1" ]; then \
        apt-get update && apt-get install -y --no-install-recommends python3 python3-pip && \
        rm -rf /var/lib/apt/lists/* && \
        pip3 install --no-cache-dir 'uniffi-bindgen==0.28.*'; \
    fi

WORKDIR /build

# Copiar a árvore real de uma vez, sem fabricar stubs de diretórios antigos:
# o layout atual é `core/src/lib.rs` + `core/src/openmtr/**`.
# O cache do registro Cargo é montado, sem criar arquivos que sombreiem o código.
COPY . .

# Cada crate é construído com `--locked`, respeitando o Cargo.lock do repo.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --locked -p open-mtr-cli
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --locked -p open-mtr-napi
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --locked -p open-mtr-uniffi
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo test --locked -p open-mtr-core

# Relatório de artefatos e geração opcional de bindings Python.
# O UDL real é `uniffi/src/open_mtr.udl`; a lib é `libuniffi_open_mtr.so`
# (nome definido por `[lib] name` em uniffi/Cargo.toml).
RUN mkdir -p /out/bindings
RUN set -eu; \
    echo "=== binários ==="; \
    ls -l target/debug/openmtr; \
    echo "=== bibliotecas ==="; \
    ls -l target/debug/libopen_mtr_napi.so target/debug/libuniffi_open_mtr.so; \
    if [ "$ENABLE_PYTHON_BINDINGS" = "1" ]; then \
        mkdir -p /out/bindings && \
        cp target/debug/libuniffi_open_mtr.so /out/bindings/ && \
        uniffi-bindgen generate uniffi/src/open_mtr.udl \
            --language python --out-dir /out/bindings/ && \
        echo "Bindings Python gerados em /out/bindings"; \
    fi

#_runtime
FROM debian:bookworm-slim

# CA certificates para TLS (reqwest/rustls usa a store do sistema via
# rustls-native-certs/webpki-roots). curl apenas para o smoke-test do healthcheck.
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Somente a CLI e os artefatos das bibliotecas — nenhum toolchain/cargo home.
COPY --from=builder /build/target/debug/openmtr /usr/local/bin/openmtr
COPY --from=builder /build/target/debug/libopen_mtr_napi.so /opt/open-mtr/
COPY --from=builder /build/target/debug/libuniffi_open_mtr.so /opt/open-mtr/
COPY --from=builder /out/bindings/ /opt/open-mtr/bindings/

# Verificação honesta: a imagem falha em partir se a CLI não responder.
RUN openmtr --version && openmtr --help >/dev/null

# Relata as bibliotecas construídas (e os bindings Python, se habilitados).
COPY <<'REPORT' /usr/local/bin/openmtr-report
#!/bin/sh
set -eu
echo "openMTR — artefatos da imagem"
openmtr --version
echo "Bibliotecas em /opt/open-mtr:"
ls -lh /opt/open-mtr/*.so
if [ -d /opt/open-mtr/bindings ] && [ -n "$(ls -A /opt/open-mtr/bindings 2>/dev/null)" ]; then
    echo "Bindings Python em /opt/open-mtr/bindings:"
    ls -lh /opt/open-mtr/bindings
else
    echo "Bindings Python: não gerados (use --build-arg ENABLE_PYTHON_BINDINGS=1)."
fi
REPORT
RUN chmod +x /usr/local/bin/openmtr-report

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD openmtr --version >/dev/null || exit 1

CMD ["openmtr", "--help"]
