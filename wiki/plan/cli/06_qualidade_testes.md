# 06 — Estratégia de testes, fixtures e gates de qualidade

> **Status:** etapa 6 (qualidade). **Depende de:** arquitetura, superfície, dump/batch e
> determinismo. **Alimenta:** distribuição e operação.
>
> Regra: **nenhum teste de CI depende de credencial real**. Live tests continuam opt-in.

## 1. Pirâmide

```text
                    ┌──────────────┐
                    │ smoke install│  (release, 3 SOs)
                    └──────┬───────┘
                 ┌─────────┴─────────┐
                 │ integração CLI     │  assert_cmd + wiremock + golden
                 └─────────┬─────────┘
          ┌────────────────┴────────────────┐
          │ unidades de config/accounts/dump │  puro, sem rede
          └────────────────┬────────────────┘
                    ┌──────┴──────┐
                    │ core tests  │  já existentes
                    └─────────────┘
```

## 2. Unidades puras

### 2.1 Config

Arquivos de fixture em `cli/tests/fixtures/config/`:

| Fixture | Esperado |
|---|---|
| `minimal.toml` | uma conta token válida |
| `multi-same-system.toml` | duas contas `mtr_pe` com CNPJs diferentes |
| `all-auth-types.toml` | token, cpf_senha_unidade, conta disabled |
| `duplicate-id.toml` | erro `config` com id duplicado |
| `missing-cnpj-pe.toml` | erro `config` para MTR-PE |
| `unknown-system.toml` | erro `config`/schema |
| `bad-url.toml` | erro de schema para `base_url` |
| `secret-both.toml` | `token` + `token_env` ⇒ erro |
| `env-missing.toml` | env ausente ⇒ erro online, warning offline |
| `legacy-v0.toml` | migração v0 → v1 (quando existir) |

Testes:

- `config_accepts_multi_same_system`
- `config_rejects_duplicate_id`
- `config_rejects_missing_cnpj_for_mtr_pe`
- `config_resolves_env_secret_and_redacts`
- `config_inline_secret_warns`
- `config_file_permissions_unix`
- `config_migrate_v0_to_v1`

### 2.2 Accounts/escopo

- `resolve_account_exact_by_id`
- `resolve_system_selects_all_enabled`
- `resolve_tag_intersects_system`
- `resolve_all_accounts_excludes_disabled`
- `resolve_ambiguous_read_requires_scope`
- `resolve_write_rejects_multi_account`
- `resolve_dump_without_scope_is_invalid`
- `resolve_account_conflicts_with_system`
- `resolve_order_is_stable_by_id`

### 2.3 Dump

- `windows_split_30_days` (inclusive, bordas)
- `windows_invalid_range`
- `normalize_mtr_sinir_fixture`
- `normalize_mtr_mtrpe_fixture`
- `aggregate_residuos_rounding`
- `merge_detail_preserves_parts`
- `pdf_filename_sanitized`
- `resume_state_roundtrip`
- `resume_rejects_different_period`

### 2.4 Batch

- `batch_parses_jsonl`
- `batch_rejects_unknown_op`
- `batch_rejects_invalid_params`
- `batch_dry_run_no_network`
- `batch_continue_on_error`
- `batch_preserves_input_order`
- `batch_rejects_json_flag`

## 3. Integração CLI com wiremock

Usar `wiremock` com `base_url` da conta no config de teste:

```toml
[[accounts]]
id = "sinir-test"
system = "sinir"
[accounts.auth]
type = "token"
token = "test-token"
```

O `openmtr` real (via `assert_cmd`) usa `--config` apontando para arquivo temporário cujo
`base_url` é `server.uri()`. Nenhum mock de transporte interno é necessário.

### 3.1 Casos por comando

| Comando | Fixture HTTP | Asserts |
|---|---|---|
| `config validate/show/accounts` | — | stdout/envelope, exit 0/2/7 |
| `account list/test` | `/gettoken` + leitura barata | auth, redaction, exit 10 |
| `mtr consultar` | lista/detalhe | filtros, paginação, `--params` |
| `mtr criar/receber/cancelar/modificar/aceitar/complementar` | respostas de sucesso/erro | payload enviado, `ok=false`, exit 1 |
| `mtr download` | PDF com magic bytes válidos/inválidos | arquivo criado, hash, erro |
| `cdf consultar/gerar/download` | respostas | payload e arquivos |
| `ref listar` | arrays desordenados | saída ordenada |
| `health` | leitura barata | `ok`, latência omitida no determinístico |
| `dump` | SINIR + 2 MTR-PE | JSONL/CSV/relatório golden |
| `batch` | respostas variadas | JSONL por linha, ordem, exit |
| `self version/completions` | — | stdout/exit |

### 3.2 Casos de entrada

- `--params` inline.
- `--params -` com JSON por stdin.
- heredoc (`assert_cmd` com `.write_stdin`).
- `--params` inválido: exit 8, mensagem sem eco do conteúdo inteiro.
- `--params` + flags conflitantes: exit 2.
- stdout pipe fechado: `assert_cmd` com `Stdio::piped` e leitura parcial; exit 0.
- `--quiet`, `--log-level off`, `--json`.
- `NO_COLOR`/`--no-color`.

## 4. Golden tests

Diretório:

```text
cli/tests/golden/
├── config/
│   ├── validate.json
│   └── accounts.txt
├── mtr/
│   ├── consultar.json
│   └── consultar.jsonl
├── ref/
│   └── classes.txt
├── dump/
│   ├── mtrs.jsonl
│   ├── mtrs.csv
│   ├── residuos.csv
│   └── relatorio.json
└── batch/
    ├── resultado.jsonl
    └── dry-run.jsonl
```

Regras:

- Golden nunca contém segredo real; usar tokens `test-*` e CPF/CNPJ fictícios.
- Sempre rodar com `--fixed-time 2026-06-01T00:00:00Z` quando houver UUID/data.
- Comparar bytes (incluindo `\n`), com normalização apenas de caminhos temporários.
- Mudou contrato? Atualizar golden no mesmo PR + nota em `wiki/plan/cli/` e `CHANGELOG`.

## 5. Testes de segurança

- `no_secret_in_stdout`: config com token/senha fake; capturar stdout/stderr; assert ausência.
- `redact_authorization_header`: erro HTTP 401 com header fake; assert ausência.
- `redact_params_error`: JSON inválido contendo `"senha":"fake"` não ecoa o valor.
- `config_show_redacts_inline_secret`.
- `config_file_mode_0600` (Unix).
- `out_path_no_traversal`: numero `../../x` vira nome sanitizado.
- `resume_state_no_secret`.

## 6. Testes de cross-platform

| Plataforma | O que testa |
|---|---|
| Linux | suíte completa, permissões 0600, paths XDG |
| macOS | suíte completa, paths `$HOME/.config`, binário arm64/x86_64 |
| Windows | paths `%APPDATA%`, ausência de checagem de permissão Unix, `install.sh` via Git Bash |
| Todos | `self completions` gera shell correto; `--config` com path absoluto/relativo |

Testes `#[cfg(unix)]` para modo; não falhar no Windows por ausência de `std::os::unix`.

## 7. Testes de instalação/release

- `scripts/package.sh` local: gera arquivo com nome `openmtr-<versão>-<target>.{tar.gz,zip}` e
  manifesto `sha256sums.txt`.
- `install.sh` com `OPENMTR_BASE_URL` apontando para um servidor HTTP local (fixture) e
  `OPENMTR_TARGET`/`OPENMTR_VERSION` fixos:
  - baixa, verifica checksum, instala em diretório temporário;
  - checksum inválido aborta;
  - upgrade preserva config do usuário;
  - `--dry-run` não escreve.
- Smoke de binário instalado: `openmtr self version` responde.
- CI: matrix de release compila, mas testes de instalação rodam localmente/smoke em Linux.

## 8. Gates de CI

### 8.1 Job `quality`

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
make file-length
make doc
```

### 8.2 Job `cli`

```bash
cargo test -p open-mtr-cli
cargo test -p open-mtr-cli --test golden
cargo test -p open-mtr-cli --test security
```

### 8.3 Job `supply-chain`

```bash
cargo deny check
cargo machete
typos
# cargo audit quando disponível
```

### 8.4 Job `cross-platform`

- `cargo test -p open-mtr-core` (já existe)
- `cargo test -p open-mtr-cli` em Linux/macOS/Windows (testes sem rede)
- `cargo check -p open-mtr-cli --target <target>` para targets da release

### 8.5 Job `release`

Ver [`07_distribuicao_ci_cd.md`](07_distribuicao_ci_cd.md).

## 9. Matriz de aceite por marco

| Marco | Testes mínimos | Critério |
|---|---|---|
| M0 Fundação | config + accounts + output + exit | `make check`; fixtures de config; envelope golden |
| M1 Operações | wiremock de cada `mtr`/`cdf`/`ref`/`health` | todos os comandos têm teste de sucesso e erro |
| M2 Dump | golden de dump SINIR + 2 MTR-PE; resume; PDF | bytes estáveis; sem segredo |
| M3 Batch | JSONL, dry-run, continue-on-error, ordem | exit e linhas golden |
| M4 Distribuição | package/install smoke + release matrix | binário executa em cada target; checksum válido |

## 10. Definition of Done da qualidade

- [ ] `make check` verde.
- [ ] `make ci` verde (deny/machete/typos; audit se disponível).
- [ ] `make doc` verde.
- [ ] Nenhum arquivo de produção >300 linhas; `scripts/check_file_length.sh` inclui `cli`.
- [ ] Nenhum `unwrap`/`expect`/`panic`/`unsafe` em `cli/src`.
- [ ] Nenhum segredo em fixture/golden/log/artefato.
- [ ] Todos os comandos documentados têm teste de sucesso + erro + exit code.
- [ ] Golden cobre envelope, dump, batch e config.
- [ ] Teste de EPIPE e stdin/heredoc.
- [ ] Matriz cross-platform verde no CI.

## 11. Riscos de teste

| Risco | Mitigação |
|---|---|
| wiremock não reproduz rate limit/retry real | testes de retry ficam no core; CLI testa apenas erros finais |
| golden instável por UUID/timestamp | `--fixed-time`; sem `generated_at`/`duration_ms` no modo determinístico |
| fixture MTR-PE grande demais | sintética e mínima; uma amostra real anonimizada por sistema |
| testes de permissão quebram no Windows | `#[cfg(unix)]`; job Windows só paths/CLI |
| `assert_cmd` não simula TTY | testar `--params -` explicitamente; não depender de `is_terminal` na v1 |
| release matrix cara | cache de cargo, artefatos curtos, matriz nativa como no knudge |
