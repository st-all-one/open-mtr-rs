# 02 — Arquitetura da CLI, crate e integração com o core

> **Status:** etapa 2 (arquitetura). **Depende de:** `01_contrato_config.md`.
> **Alimenta:** superfície, dump/batch, testes e distribuição.

## 1. Decisão de workspace

Criar um novo membro no workspace:

```text
open-mtr-rs/
├── core/                 # open-mtr-core (inalterado como domínio)
├── cli/                  # open-mtr-cli (novo)
├── napi/
├── uniffi/
├── wasm/
├── wiki/plan/cli/
├── install.sh            # novo, raiz do repo
└── scripts/package.sh    # novo, empacotamento local
```

`Cargo.toml` do workspace:

```toml
members = ["core", "cli", "napi", "uniffi", "wasm"]
default-members = ["core", "cli"]
```

Novo crate:

```toml
[package]
name = "open-mtr-cli"
edition.workspace = true
version.workspace = true
rust-version.workspace = true
license.workspace = true
description.workspace = true
repository.workspace = true
publish.workspace = true

[[bin]]
name = "openmtr"
path = "src/main.rs"

[dependencies]
open-mtr-core = { path = "../core" }
# borda: clap, serde, serde_json, toml/toml_edit, tracing, tracing-subscriber
# runtime: tokio com rt-multi-thread/macros/fs/io-util/time
# saída: csv (dump), sem reqwest direto
```

**Nome do binário:** `openmtr` (curto para uso diário, sem colidir com `open-mtr` de pacote).
**Nome do config:** `open-mtr-config.toml` (explícito e estável).
**Feature:** o CLI compila sempre com `open-mtr-core/native-http`; não é alvo WASM.

## 2. Princípios de camadas

```text
main.rs
  └─ cli/ (clap)                    # parsing, validação de flags, help
      └─ commands/                  # orquestração de um verbo
          ├─ config/                # carregar, validar, migrar, redigir
          ├─ accounts/              # resolver escopo, autenticar, criar clientes
          ├─ dump/                  # pipeline de dump multi-conta
          ├─ batch/                 # executor JSONL
          └─ output/                # envelope, texto, JSONL, CSV
              └─ open_mtr_core      # domínio, clientes, adapters, erros
```

Regras:

1. `commands/*` não conhece detalhes de HTTP nem de adapter; só chama `OpenMtrClient`.
2. `accounts/*` é a única camada que traduz `AccountConfig` em `OpenMtrClient`.
3. `config/*` é puro: lê TOML, valida, resolve segredos, não faz rede.
4. `dump/*` pode orquestrar várias contas e arquivos, mas usa `OpenMtrClient` para tudo.
5. `output/*` é a única camada que escreve em stdout/stderr.

## 3. Módulos previstos

```text
cli/src/
  main.rs                    # entrypoint: parse -> logging -> dispatch -> exit
  cli.rs / cli/              # Cli, Command, Args, enums fechados
  error.rs                   # CliError, kind, exit code, retryable, redaction
  logging.rs                 # tracing-subscriber em stderr, nível/quiet
  output/mod.rs              # emit_stdout/emit_stderr, EPIPE, envelope
  output/envelope.rs         # {success, command, data, error, warnings}
  output/jsonl.rs            # streaming JSONL
  output/csv.rs              # CSV estável (dump)
  config/mod.rs              # ConfigFile, AccountConfig, Defaults
  config/paths.rs            # resolução de --config/env/XDG/APPDATA
  config/schema.rs           # tipos serde + validação estrutural
  config/secrets.rs          # token_env/senha_env, redaction, permissions
  config/validate.rs         # regras de negócio da config
  config/migrate.rs          # config_version -> versão atual
  accounts/mod.rs            # AccountRef, Scope, resolução
  accounts/resolve.rs        # --account/--system/--tag/--all-accounts
  accounts/runtime.rs        # OnceCell<Arc<OpenMtrClient>> por conta
  accounts/auth.rs           # token vs cpf_senha_unidade -> OpenMtrClient
  commands/mod.rs            # dispatch
  commands/mtr.rs            # consultar/criar/receber/cancelar/modificar/...
  commands/cdf.rs            # consultar/gerar/download
  commands/ref.rs            # tabelas de referência
  commands/health.rs         # health
  commands/config_cmd.rs     # init/validate/show/accounts/edit/migrate
  commands/dump.rs           # entrypoint do dump
  dump/mod.rs                # orquestração
  dump/model.rs              # MtrNormalizada + agregados
  dump/normalize.rs          # core Mtr -> modelo de saída
  dump/windows.rs            # janelas de período
  dump/output.rs             # jsonl/csv/report
  dump/pdf.rs                # download/resume de PDFs
  dump/resume.rs             # manifesto de retomada
  batch/mod.rs               # entrypoint do batch
  batch/spec.rs              # linha JSONL (op, account, params, out)
  batch/run.rs               # execução, dry-run, continue-on-error
  self_cmd.rs                # version/completions/update wrapper
  tests/                     # integração, golden, fixtures
```

Arquivos de produção ≤300 linhas. Se `dump/normalize.rs` passar disso, dividir em
`normalize/partes.rs`, `normalize/residuos.rs`, `normalize/agregados.rs`.

## 4. Construção de cliente por conta

### 4.1 Conta com `auth.type = "token"`

```text
AccountConfig { system, base_url, token_env|token, timeout, max_concurrency }
  -> OpenMtrClient::builder()
       .base(system)
       .base_url(base_url.unwrap_or_default())
       .token(resolved_token)
       .timeout_secs(...)
       .max_rps(max_concurrency)
       .clock/clock fixo (se modo determinístico)
       .id_generator/gerador determinístico
       .build()
       .await
```

O core já faz `adapter.autenticar_token` no `build()`.

### 4.2 Conta com `auth.type = "cpf_senha_unidade"`

```text
AccountConfig { system, base_url, cpf, cnpj, unidade, senha_env|senha }
  -> OpenMtrClassicAuth::new(system)
       .base_url(base_url.unwrap_or_default())
       .params(AuthParams::CpfSenhaUnidade { cpf, senha, unidade, cnpj })
       .timeout_secs(...)
       .build()
       .await
  -> token JWT
  -> OpenMtrClient::builder().base(system).token(token).build()
```

Observações:

- MTR-PE exige `cnpj`; o validador da config rejeita antes.
- SIGOR ignora `cnpj`; aceitar `cnpj = None` ou preenchido.
- O JWT é mantido pela `TokenManager` do cliente; não é persistido em disco.
- `OpenMtrClassicAuth` não recebe clock/IdGenerator; isso não afeta a saída de dados, só o UUID
  interno do erro de auth. O `OpenMtrClient` construído recebe clock/id fixos quando aplicável.

### 4.3 Cache de clientes

Cada `AccountRuntime` contém:

```rust
struct AccountRuntime {
    config: AccountConfig,
    client: tokio::sync::OnceCell<Arc<OpenMtrClient>>,
}
```

- `client()` autentica/cria uma única vez por processo.
- Em `dump --all-accounts`, cada conta tem seu runtime; tarefas concorrentes compartilham o mesmo
  `Arc<OpenMtrClient>` da conta, respeitando o `max_concurrency` do core.
- Em comandos de uma conta, o runtime é descartado ao final do processo.
- Não persistir JWT em disco na v1 (evita cache/segredo fora do core); rate limit do core cobre o
  custo.

### 4.4 Escopo de seleção

`accounts::resolve(scope, config)` retorna `Vec<Arc<AccountRuntime>>` **ordenado por id**. Regras
do arquivo 01 são aplicadas antes de criar qualquer cliente. Erros de escopo são `InvalidInput` e
não tocam a rede.

## 5. Modelo de erro e exit codes

```rust
pub enum CliError {
    InvalidInput(String),
    NotFound(String),
    Conflict(String),
    Io { context: String, source: std::io::Error },
    Config(String),
    Schema(String),
    Unsafe(String),
    Timeout(String),
    Auth(String),
    Upstream(OpenMTRError),
    Business { status: u16, message: String },
    Internal(String),
}
```

Mapeamento inicial:

| Código | Kind | Origem |
|---:|---|---|
| 0 | sucesso | comando concluído, inclusive `--dry-run` |
| 1 | parcial | `Resposta { ok: false }` ou lote com falhas parciais |
| 2 | invalid input | clap, escopo ambíguo, `--params` inválido, flag conflitante |
| 3 | not found | `OpenMTRError::NotFound`, arquivo/PDF ausente |
| 4 | conflict | conta duplicada, arquivo de saída existente sem `--force` |
| 5 | io | escrita de arquivo, stdout/stderr não-EPIPE |
| 6 | timeout | `OpenMTRError::Timeout` |
| 7 | config | config ausente/inválida, env de segredo ausente |
| 8 | schema | TOML/JSON/JSONL malformado, versão incompatível |
| 9 | unsafe | permissão de config com segredo inline em modo strict |
| 10 | auth | `OpenMTRError::Auth` |
| 11 | upstream/api | `OpenMTRError::Api`/5xx persistente |
| 70 | internal | invariante quebrada, bug |
| 101 | panic | reservado (não esperado) |

EPIPE em stdout ⇒ exit 0 (contrato `kd`). `strict = true` promove warnings de config/segredo a
erro apropriado.

## 6. Determinismo e relógio

- O CLI aceita `--fixed-time <RFC3339>` (global, oculto no help comum ou documentado em
  `openmtr help determinism`) e/ou `OPENMTR_FIXED_NOW`.
- Com `--fixed-time`, cria `FixedClock` e `SequentialIdGenerator` e injeta em cada
  `OpenMtrClientBuilder`.
- `dump` só inclui `generated_at` quando **não** está em modo determinístico; em modo determinístico o
  relatório usa `generated_at = fixed_time` ou omite o campo (decidir na etapa 05).
- Ordenação de contas e saídas é sempre explícita (`id`, `system`, `numero`, `codigo`).
- Não usar `HashMap` para ordem de saída; usar `BTreeMap`/`Vec` ordenada.

## 7. Runtime assíncrono

- `#[tokio::main(flavor = "multi_thread")]` no binário; operações são I/O-bound.
- `tokio` features: `macros`, `rt-multi-thread`, `fs`, `io-util`, `time`, `sync`.
- O core já tem `reqwest`/`tokio` via `native-http`; o CLI não adiciona cliente HTTP.
- `--concurrency` controla o fan-out do dump e é limitado por conta/global; nunca `join_all`
  irrestrito.
- Cancelamento: `Ctrl-C` deve abortar tarefas, fechar arquivos de saída de forma consistente e
  deixar o manifesto de retomada válido. Não usar `panic` para isso.

## 8. Dependências e orçamento

| Dependência | Uso | Justificativa |
|---|---|---|
| `clap` (derive) | parsing/help | padrão do ecossistema; já usado no knudge |
| `clap_complete` | `self completions` | geração oficial de completions |
| `toml` | leitura tipada da config | evita parser próprio |
| `toml_edit` | migração/reescrita preservando comentários | atualização prática de config |
| `serde`/`serde_json` | params/envelope/JSONL | já no workspace |
| `tracing`/`tracing-subscriber` | logs stderr, níveis | borda observável |
| `tokio` | runtime assíncrono | core já depende |
| `csv` | CSV de dump | escaping correto em vez de manual |
| `chrono` | datas/janelas | já no core |
| `sha2` | hash de PDF/artefatos | manifesto verificável; licença compatível |

Dev: `wiremock` (já no core), `assert_cmd`, `predicates`, `tempfile`. Nenhuma dependência de HTTP
direta no CLI. `cargo deny`/`machete` devem passar.

## 9. Alterações esperadas no core

Preferência: **nenhuma alteração de domínio**. Alterações pequenas aceitáveis, se necessárias:

1. Expor um helper para validar sistema suportado (`System::has_adapter()`), em vez de instanciar
   adapter na validação de config.
2. Garantir que `OpenMtrClientBuilder` aceite `Clock`/`IdGenerator` (já aceita) e que o CLI consiga
   usar isso em todas as contas.
3. Se o dump precisar de campos ainda não expostos, adicionar somente leitura no domínio com teste
   de round-trip; nunca regra de apresentação no core.
4. Não mover normalização de dump para o core: ela é contrato do CLI (CSV/JSONL).

## 10. Fluxo de execução

```text
main
  -> parse clap
  -> logging (stderr; quiet/off)
  -> config::load + validate estrutural
  -> dispatch:
       config/self  -> sem conta
       comandos     -> accounts::resolve + runtime.client()
       dump         -> resolve N contas -> pipeline por conta -> merge -> output
       batch        -> lê JSONL -> resolve conta por linha -> runner
  -> output::emit_success/emit_error
  -> exit code
```

## 11. Testabilidade

- `config` puro: unit tests com TOML strings.
- `accounts` puro: resolve scope com fixtures, sem rede.
- `commands` com `wiremock`: o config de teste aponta `base_url` para `server.uri()` e o binário
  real é exercitado via `assert_cmd`. Não é necessário expor transporte injetável no CLI; se algum
  caso precisar de controle fino, um construtor interno test-only pode existir sem entrar na
  superfície pública.
- `dump` com fixtures JSON e `MockTransport`; saída golden.
- `batch` com JSONL de fixture e operações mockadas.
- `self/install` com script fake e asserts de argumentos, sem rede.

## 12. Riscos arquiteturais

| Risco | Mitigação |
|---|---|
| CLI virar “segundo core” | regra de camadas + revisão: nenhum adapter/HTTP no CLI |
| Clientes multi-conta vazarem estado | runtime por conta, sem default global; testes de escopo |
| `toml_edit`/`toml` duplicarem parsing | um schema serde; `toml_edit` só para escrita/migração |
| Dump ficar grande demais | módulos ≤300 linhas; pipeline puro e testável |
| Runtime pesado | tokio multi-thread é necessário; sem `tokio full` |
| Testes dependerem de rede | `HttpTransport` injetável e wiremock; live tests opt-in |
