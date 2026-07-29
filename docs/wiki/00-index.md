# mtr-sinir-core — Índice

Pacote Rust de integração com a API SINIR (Sistema Nacional de Informações sobre a Gestão dos Resíduos Sólidos).

## Stack

| Camada | Crate | Função |
|--------|-------|--------|
| Runtime | `tokio` (sync, time) | Async runtime |
| HTTP | `reqwest` 0.12 (rustls-tls) | Cliente HTTP |
| Serialização | `serde` + `serde_json` | JSON (de)serialize |
| Erros | `thiserror` | Erros tipados |
| UUID | `uuid` (v7) | Correlation ID p/ erros |
| Logs | `tracing` | Spans + `#[instrument]` |
| Cache | `moka` (opt) | Cache concorrente |

## Estrutura

```
core/src/
├── lib.rs                    # Barrel + prelude
├── client/
│   ├── mod.rs
│   ├── http.rs               # MtrClient (reqwest wrapper)
│   ├── token.rs              # TokenManager (auto-renovação)
│   ├── retry.rs              # with_retry() — backoff exp.
│   └── endpoint.rs           # trait Endpoint + PaginatedIter
├── core/
│   ├── mod.rs
│   ├── config.rs             # MtrConfig + MtrConfigBuilder + AuthMethod
│   └── errors.rs             # MtrError (RFC 7807) + ErrorCategory
├── schemas/
│   ├── mod.rs
│   ├── common.rs             # ObjetoRespostaRest<T>, DataMtr, Pagina
│   ├── auth.rs               # Login/Tokens
│   ├── cnpj.rs               # Cnpj newtype
│   ├── cpf.rs                # Cpf, CpfCnpj
│   ├── manifesto.rs          # Lista, cancelamento, TipoParceiro
│   ├── recebimento.rs        # Receber lote, alterar, aceitar
│   ├── cdf.rs                # Emitir/download CDF
│   ├── logistica_reversa.rs  # LR origem/processo
│   ├── integracao_estadual.rs# Integração estadual
│   └── tabelas.rs            # Acondicionamento + LazyLock maps
├── services/
│   ├── mod.rs
│   ├── auth.rs               # AuthService (deprecated)
│   ├── manifesto.rs          # ManifestoService + builders
│   ├── salvar_manifesto.rs   # SalvarManifestoLoteBuilder
│   ├── recebimento.rs        # RecebimentoService
│   ├── cdf.rs                # CdfService
│   ├── logistica_reversa.rs  # LogisticaReversaService
│   ├── integracao_estadual.rs# IntegracaoEstadualService
│   └── tabelas.rs            # TabelasService (cache)
└── cache/
    ├── mod.rs
    └── tabelas.rs             # Cache thread-safe com TTL
```

## Wiki Contents

| File | Topic |
|------|-------|
| [01-config-client.md](01-config-client.md) | MtrConfig + MtrClient + TokenManager |
| [02-schemas.md](02-schemas.md) | All schema types by domain |
| [03-services.md](03-services.md) | Service layer + builders |
| [04-endpoint.md](04-endpoint.md) | trait Endpoint + PaginatedIter |
| [05-errors.md](05-errors.md) | Error system (RFC 7807) |
| [06-newtypes.md](06-newtypes.md) | Cnpj, Cpf, DataMtr, Pagina, TimestampMs |
| [07-cache.md](07-cache.md) | Cache + tabelas de apoio |
| [08-testing.md](08-testing.md) | Test structure + wiremock |
