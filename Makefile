# openMTR — alvos de qualidade e build.

CARGO ?= cargo

.PHONY: check fmt clippy test build file-length clean deny audit machete typos ci doc napi uniffi python kotlin swift core update-version

## Portão local: formatação, lints, testes e orçamento de tamanho de arquivo.
check: fmt clippy test file-length

## Verifica formatação sem alterar.
fmt:
	$(CARGO) fmt --all -- --check

## Lints de todos os alvos com warnings como erro.
clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

## Testes do workspace (inclui doc-tests).
test:
	$(CARGO) test --workspace

## Build de debug do workspace.
build:
	$(CARGO) build --workspace

## Nenhum arquivo de produção cresce além do orçamento transitório.
file-length:
	./scripts/check_file_length.sh

clean:
	$(CARGO) clean

## Supply chain: licenças/advisories/fontes.
deny:
	@if command -v cargo-deny >/dev/null 2>&1; then \
		$(CARGO) deny check; \
	else \
		echo "cargo-deny ausente; pule (cargo install cargo-deny --locked)"; \
	fi

audit:
	@if command -v cargo-audit >/dev/null 2>&1; then \
		$(CARGO) audit; \
	else \
		echo "cargo-audit ausente; pule (cargo install cargo-audit --locked)"; \
	fi

## Dependências não usadas.
machete:
	@if command -v cargo-machete >/dev/null 2>&1; then \
		$(CARGO) machete; \
	else \
		echo "cargo-machete ausente; pule (cargo install cargo-machete --locked)"; \
	fi

## Verificação ortográfica.
typos:
	@if command -v typos >/dev/null 2>&1; then \
		typos; \
	else \
		echo "typos ausente; pule (https://github.com/crate-ci/typos)"; \
	fi

## Portão do CI: check + extras disponíveis.
ci: check deny audit machete typos

## Atualiza a versão do workspace: make update-version VERSION=0.2.0
update-version:
	@test -n "$(VERSION)" || { echo "uso: make update-version VERSION=0.2.0"; exit 1; }
	./scripts/bump-version.sh "$(VERSION)"
	$(CARGO) update -w

doc:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --workspace --no-deps

# --- Bindings (mantidos para compatibilidade com o fluxo existente) ---

core:
	$(CARGO) check -p open-mtr-core

napi:
	cd napi && npm ci && npm run build

uniffi:
	$(CARGO) build -p open-mtr-uniffi --release

python: uniffi
	uniffi-bindgen generate uniffi/src/open_mtr.udl --language python --out-dir bindings/python/
	cd bindings/python && pip install -e .

kotlin: uniffi
	uniffi-bindgen generate uniffi/src/open_mtr.udl --language kotlin --out-dir bindings/kotlin/

swift: uniffi
	uniffi-bindgen generate uniffi/src/open_mtr.udl --language swift --out-dir bindings/swift/
