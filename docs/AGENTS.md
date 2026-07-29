# mtr-sinir-rs — Guia do Agente

## TL;DR

```bash
cargo build
cargo test                       # 42 testes (40 unit + 2 integration)
cargo clippy                     # 0 errors
```

## Stack

| Componente | Tecnologia |
|---|---|
| Runtime | Rust 1.85+ (edition 2024) |
| Async | `tokio` (sync, time) |
| HTTP | `reqwest` 0.12 (async, rustls-tls) |
| Serialização | `serde` + `serde_json` |
| Erros | `thiserror` — RFC 7807 com ErrorCategory (10) + UUID v7 |
| Logs | `tracing` — spans, `#[instrument]`, structured |
| UUID | `uuid` v7 (correlação distribuída) |
| Rate limiting | Sliding window com `tokio::sync::Mutex` |
| Retry | `client::retry::with_retry` — backoff exponencial (3 tries, 1s/2s/4s) |

## Arquitetura

```
mtr_sinir (lib)
├── lib.rs              # Barrel + prelude
├── client/
│   ├── mod.rs
│   ├── http.rs         # MtrClient (reqwest wrapper, auth, rate-limit)
│   ├── token.rs        # TokenManager (auto-renovação, cache, expiração)
│   ├── retry.rs        # with_retry() — backoff exponencial
│   └── endpoint.rs     # trait Endpoint + PaginatedIter
├── core/
│   ├── mod.rs
│   ├── config.rs       # MtrConfig + MtrConfigBuilder + AuthMethod
│   └── errors.rs       # MtrError (enum), ErrorCategory (10), ErrorContext, UUID v7
├── schemas/            # Tipos serde (1 módulo por domínio)
│   ├── mod.rs          # Barrel
│   ├── common.rs       # ObjetoRespostaRest<T>, DataMtr, Pagina, TamanhoPagina
│   ├── auth.rs         # ApiwsLoginDTO, TokenResponse
│   ├── cnpj.rs         # Cnpj newtype (+ inline tests)
│   ├── cpf.rs          # Cpf, CpfCnpj (+ inline tests)
│   ├── manifesto.rs    # ApiwsListaManifestoDTO, TipoParceiro, + 9 sub-structs
│   ├── recebimento.rs  # ApiwsManifestoDeRecebimentoDTO, TimestampMs
│   ├── cdf.rs          # ApiwsCertificadoDestinacaoDTO, FormatoDownload
│   ├── logistica_reversa.rs  # MtrLrDataDTO, RastreioEvento
│   ├── integracao_estadual.rs  # ParteMTR (Cnpj), ResiduoIntegracao
│   └── tabelas.rs      # TipoAcondicionamento + LazyLock maps
├── services/           # Lógica de negócio (1 módulo por domínio)
│   ├── mod.rs
│   ├── auth.rs
│   ├── manifesto.rs    # ManifestoService + ManifestoListBuilder (typestate)
│   ├── salvar_manifesto.rs  # SalvarManifestoLoteBuilder (typestate)
│   ├── recebimento.rs
│   ├── cdf.rs
│   ├── logistica_reversa.rs
│   ├── integracao_estadual.rs
│   └── tabelas.rs      # Cache 24h via RwLock
└── cache/
    ├── mod.rs
    └── tabelas.rs       # Cache thread-safe com RwLock ou moka
```

## Convenções de código

- `snake_case` para funções e variáveis; `CamelCase` para tipos
- Todos os tipos de schema: `Debug, Clone, Serialize, Deserialize`
- Campos opcionais: `Option<T>` com `#[serde(skip_serializing_if = "Option::is_none")]`
- Métodos públicos retornam `Result<T, MtrError>` (async)
- Identificador de operação (ex: `"manifesto.listar"`) para tracing
- `unwrap()` proibido em produção (use `?` ou `expect()` com mensagem)
- `#[must_use]` em construtores e funções de validação
- `#[non_exhaustive]` em enums públicos
- Newtypes: `::new()` valida, `Deserialize` leniente

## Dependências (prod)

| Crate | Motivo |
|---|---|
| `tokio` | Runtime async (sync, time) |
| `reqwest` | HTTP client async com TLS |
| `serde` + `serde_json` | Serialização JSON |
| `thiserror` | Macro de erro derive |
| `tracing` | Logging estruturado com spans |
| `uuid` (v7) | Correlation IDs para erros |
| `chrono` | Manipulação de datas/horários |
| `url` | Manipulação de URLs |
| `moka` (opt) | Cache concorrente para produção |

## Testes

```bash
cargo test                          # 42 testes (40 unit inline + 2 wiremock)
cargo test --test integration_test  # só wiremock (2 testes)
cargo clippy                        # 0 errors
```

Testes unitários de schema vivem em `#[cfg(test)] mod tests` dentro de cada
arquivo de schema em `core/src/schemas/`. Testes de integração (wiremock)
ficam em `core/tests/integration_test.rs`.

## Newtypes com Validação

| Tipo | Validação | `Deserialize` |
|------|-----------|---------------|
| `Cnpj` | 14 dígitos + check digits | Leniente |
| `Cpf` | 11 dígitos + check digits | Leniente |
| `CpfCnpj` | 11→Cpf, 14→Cnpj | Automático |
| `DataMtr` | `dd-MM-yyyy` | Parse |
| `Pagina` | `>= 1` | — |
| `TamanhoPagina` | `1..=50` | — |
| `TimestampMs` | ms desde epoch | i64 |

## Endpoints SINIR

| Recurso | Endpoints |
|---------|-----------|
| Auth | POST /token |
| Manifesto | POST /listaManifesto, /cancelarManifesto, /downloadManifesto/{num} |
| Recebimento | POST /receberManifestoLote, /alteraRecebimento, /aceiteAlteracaoRecebimento |
| CDF | POST /emiteCDF, /emiteCDFLR, /downloadCertificado/{cod} |
| LR Origem | POST /listaMtrOrigem, /mtrOrigemCompleto, /alterarManifestoLROrigem |
| LR Processo | POST /listaMtrProcesso, /mtrProcesso |
| IE | POST /integraMTREstadual, /integraRecebimentoMTREstadual |
| Tabelas | GET /retornaListaAcondicionamento, /retornaListaAcondicionamentoPorEstadoFisico/{cod} |

## Operações Identificadas (tracing)

| Identificador | Método |
|---|---|
| `auth.autenticar` | `AuthService::autenticar()` |
| `manifesto.listar` | `ManifestoService::listar()` |
| `manifesto.listar_todos` | `ManifestoService::listar_todos()` |
| `manifesto.cancelar` | `ManifestoService::cancelar()` |
| `manifesto.download` | `ManifestoService::download_pdf()` |
| `recebimento.registrar_lote` | `RecebimentoService::registrar_lote()` |
| `recebimento.solicitar_alteracao` | `RecebimentoService::solicitar_alteracao()` |
| `recebimento.aceitar_alteracao` | `RecebimentoService::aceitar_alteracao()` |
| `cdf.emitir` | `CdfService::emitir()` |
| `cdf.emitir_lr` | `CdfService::emitir_lr()` |
| `cdf.download` | `CdfService::download()` |
| `lr.listar_origem` | `LogisticaReversaService::listar_origem()` |
| `lr.listar_processo` | `LogisticaReversaService::listar_processo()` |
| `lr.obter_origem` | `LogisticaReversaService::obter_origem_completo()` |
| `lr.obter_processo` | `LogisticaReversaService::obter_processo()` |
| `lr.alterar_titularidade` | `LogisticaReversaService::alterar_titularidade()` |
| `integracao.integrar_manifestos` | `IntegracaoEstadualService::integrar_manifestos()` |
| `integracao.integrar_recebimento` | `IntegracaoEstadualService::integrar_recebimento()` |
| `tabelas.listar_todos` | `TabelasService::listar_todos()` |
| `tabelas.listar_por_estado` | `TabelasService::listar_por_estado_fisico()` |
