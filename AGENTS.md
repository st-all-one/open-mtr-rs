# AGENTS.md — guia de contribuição do openMTR

> Guia para agentes/contribuidores que trabalham **no código deste repositório**.
> Documentação de arquitetura: [`docs/AGENTS.md`](docs/AGENTS.md) e [`docs/wiki/`](docs/wiki/).

## 0. Regras de ouro

1. **Idioma:** documentação, comentários e commits em **português**; identificadores de código em
   **inglês** quando possível.
2. **Vermelho = não terminou.** Rode `make check` e deixe verde antes de concluir qualquer tarefa.
3. **Simplicidade e localidade** são metas globais. O core é puro; bindings são cascas finas.
4. **Domínio puro:** `open-mtr-core` não conhece APIs específicas de plataforma; o transporte é
   abstraído por `HttpTransport`.
5. **Sem `unwrap`/`expect`/`panic` em produção.** Testes e exemplos podem usar atalhos de fixture.
6. **Contrato de bytes:** serialização, nomes de campos e ordem de tabelas exigem teste/golden.

## 1. Fluxo de trabalho

```sh
make check     # fmt --check + clippy -D warnings + test + orçamento de linhas
make fmt       # cargo fmt --all
make clippy    # cargo clippy --workspace --all-targets -- -D warnings
make test      # cargo test --workspace
make build     # cargo build --workspace
make ci        # check + deny + audit + machete + typos
make doc       # rustdoc com -D warnings
```

- Toolchain: Rust **1.100+** (Edition 2024). MSRV **obrigatória** = 1.100.
  `rust-toolchain.toml` fixa o channel; `rust-version`/`clippy.toml::msrv` casam com ele.
- `rustfmt`: largura 100, indent 4, newline Unix, `style_edition = "2024"`.
- **Licenciamento:** `MIT OR Apache-2.0` (ver `LICENSE-MIT`, `LICENSE-APACHE` e
  `[workspace.package].license`). Todo crate herda a licença do workspace.

## 2. Mapa do repositório

```
core/                 # núcleo puro: domínio, adapters, transporte, cliente HTTP
cli/                  # binário `openmtr`: adaptador fino, config multi-conta, dump e batch
napi/                 # binding Node.js (napi-rs)
uniffi/               # binding Swift/Kotlin/Python (UniFFI)
wasm/                 # binding Deno/Web (wasm-bindgen)
docs/                 # documentação de arquitetura e wiki
docs-rust/            # documentação histórica (não confiável para a API atual)
wiki/                 # referências dos sistemas externos (SINIR, MTR-PE, SIGOR)
.agents/skills/rust/  # skill de Rust 1.100 (consultar ao escrever/revisar código)
scripts/              # utilitários de qualidade
```

## 3. Qualidade

- Os níveis de lint ficam em `[workspace.lints]` no `Cargo.toml`; a configuração do Clippy fica em
  `clippy.toml`. Não adicione `#![allow]` sem `reason`.
- `deny.toml` cobre advisories, licenças, wildcards e fontes. O padrão de licença é
  **MIT OR Apache-2.0**.
- O orçamento de tamanho de arquivo é **300 linhas por arquivo de produção**, aplicado por
  `scripts/check_file_length.sh` (core, cli, napi, uniffi, wasm). Não suba o teto: extraia um
  módulo coeso.

## 4. Revitalização (Rust 1.100)

Estado das dívidas conhecidas — todas fechadas; ficam registradas para não voltarem:

1. **[feita]** Adapters divididos em diretórios coesos: `adapters/{sinir,sigor,mtr_pe}/` (cada
   arquivo ≤300 linhas). O trait `MtrAdapter` fica em `adapters/trait_def.rs`; cada adapter tem
   um único `impl MtrAdapter` em `mod.rs` delegando para métodos inerentes `*_impl` nos
   submódulos. **Não** se divide `impl Trait for Type` entre arquivos (E0119).
2. **[feita]** Sem `expect`/`unwrap` em produção. Restam só os de `#[cfg(test)]` e um `expect`
   justificado em `error/types.rs::make_urn` (URN de literais infalível).
3. **[feita]** `missing_docs = "warn"` e toda a API pública (core + 4 bindings) documentada;
   `make doc` roda `RUSTDOCFLAGS=-D warnings`. A única exceção é `#[allow(missing_docs)]`
   justificado em `napi/src/lib.rs` (código gerado por `#[napi]`).
4. **[feita]** `domain/tabela.rs` usa `BTreeMap` (ordem determinística).
5. **[feita]** `wasm/src/lib.rs` usa `RefCell` com `reason` no `allow` (limite single-thread).
6. **[feita]** `Dockerfile` nos paths atuais, com job `docker` no CI.
7. **[feita]** `cli/` herda `[lints] workspace = true`; flags booleanas viraram
   `enum AccountScope`/`LogInit`, e `print_stdout`/`print_stderr` são `#[allow]` com `reason`
   nos pontos de saída. Para clap, o doc-comment do campo É o `--help`: ao mudá-lo, rode
   `make cli-docs`.
8. **[feita]** Todo arquivo de produção ≤300 linhas (cliente HTTP, domínio, erro, cliente,
   `cli`) por extração em submódulos coesos.
9. **[aberta]** `chrono` é unmaintained (RUSTSEC-2026-0543), sem correção; migrar para
   `time`/`jiff` quando houver janela. Enquanto isso, o `ignore` está justificado no
   `deny.toml` e registrado em `kd`.

## 5. Testes

```sh
cargo test -p open-mtr-core
cargo test -p open-mtr-core --test domain_test
cargo test -p open-mtr-core --test wiremock_sinir
cargo test -p open-mtr-core --test live_sinir_test   # requer .env / token real
```

Testes `live_*` dependem de credenciais e da API externa; os testes `wiremock_*` são a suíte
hermética.

<!-- knudge:start -->
<!-- knudge:version: 4 -->
# knudge — project memory

`notas/` is the source of truth; index/graph/embeddings are **derived** — never edit `notas/` by hand.

Loop: `kd prime` (protocol; `--long` = full) → `kd ask "<q>"` before acting → `kd write` each
learning → `kd rewind` for handoff (`--with-activity` adds recent agent activity).
- `kd ask` dedups: score <0.75 create · 0.75–0.92 merge · ≥0.92 reject.
- One assertion per note; anchor code with `--anchor PATH` (glob `src/**`).
- Work lives in `kd task`: `epic ⊃ {issue ⊃ task | task}`; `--scope epic|issue|task`,
`--kind task|error|question|risk|decision`; `kd task list --ready`; close with `--outcome`.
- `kd write --type task` is rejected; `ask` excludes work unless `--with-task`.
- Health: `kd doctor` (`--fix`); `kd maintenance learn|compact|prune` propose only.
- Embeddings are derived: `kd drain --status` / `--digest`.
- Agent memory shards: `kd ai --memory|--session "<q>"` (read-only NDJSON,
`.knudge/ai/<area>/<agent_id>.jsonl`); `--sort time`, `--agent-id/--agent-role`,
`--recap` (compact recap); filter `ask` with `--agent/--activity/--agent-id/--agent-role`.
- stdout = data, stderr = logs; `--json` = {success, command, data?, error?, warnings?}.
- If `kd` is missing on the host: `curl -fsSL https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh | bash` (installs `~/.local/bin`), then `kd prime`.
Config: `.knudge/config.toml`.
<!-- knudge:end -->
