# openMTR — Guia do Agente

## TL;DR

```bash
cargo build -p open-mtr-core
cargo test -p open-mtr-core                  # 96 testes
cargo clippy -p open-mtr-core -- -D warnings # 0 warnings
```

## Stack

| Componente | Tecnologia |
|---|---|
| Runtime | Rust 1.85+ (edition 2024) |
| Async | `tokio` (sync, time, fs) |
| HTTP | `reqwest` 0.12 (async, rustls-tls) |
| Serialização | `serde` + `serde_json` |
| Erros | RFC 9457 (Problem Details) + UUID v7 |
| Logs | `tracing` — spans, `#[instrument]`, structured |
| Retry | Backoff exponencial (3 tries, 1s/2s/4s) |

## Arquitetura

```
open-mtr-core (lib)
├── openmtr/
│   ├── domain/               # Tipos universais do domínio MTR
│   │   ├── mtr.rs            # Mtr, StatusMtr, FiltroMtr, Periodo, FiltroParte
│   │   ├── parte.rs          # Parte (gerador/transportador/destinador)
│   │   ├── residuo.rs        # Residuo, ResiduoCriacao, ResiduoRecebido, ResiduoIbama
│   │   ├── newtypes.rs       # Cnpj, Cpf, CpfCnpj, DataMtr, TimestampMs
│   │   ├── criacao.rs        # CriarMtr, MtrCriado
│   │   ├── recebimento.rs    # ReceberMtr, MtrRecebido
│   │   ├── cancelamento.rs   # CancelarMtr, MtrCancelado
│   │   ├── modificacao.rs    # ModificarMtr, MtrModificado
│   │   ├── download.rs       # DownloadItem, DownloadCdf
│   │   ├── cdf.rs            # GerarCdf, Cdf, CdfEmitido, FiltroCdf
│   │   ├── tabela.rs         # Acondicionamento + LazyLock maps
│   │   └── tipo.rs           # TipoParte, TipoManifesto, FormatoDownload
│   ├── adapters/             # Implementações específicas
│   │   ├── mod.rs            # trait MtrAdapter, ItemResult, LoteResult
│   │   └── sinir.rs          # SinirAdapter (~840 linhas)
│   ├── client/               # HTTP genérico
│   │   ├── http.rs           # MtrHttpClient (retry, auth, rate-limit)
│   │   └── token.rs          # TokenManager (refresh JWT)
│   ├── error.rs              # OpenMTRError (RFC 9457)
│   ├── resposta.rs           # Resposta<T>, Problema
│   ├── system.rs             # enum System
│   ├── open_mtr_client.rs    # OpenMtrClient + Builder (16 métodos)
│   └── open_mtr_auth.rs      # OpenMtrAuth (legado CPF+senha)
```

## Convenções de código

- `snake_case` para funções e variáveis; `CamelCase` para tipos
- Todos os tipos de domínio: `Debug, Clone, Serialize, Deserialize`
- Campos opcionais: `Option<T>` com `#[serde(skip_serializing_if = "Option::is_none")]`
- Métodos públicos retornam `MtrResult<T> = Result<Resposta<T>, OpenMTRError>`
- Todo método tem `tracing::info_span!` com `instance` (UUID v7), `operation`, `system`
- `unwrap()` proibido em produção (use `?` ou `expect()` com mensagem)
- `#[non_exhaustive]` em enums públicos

## Adapter Pattern

Cada sistema implementa `trait MtrAdapter` (16 métodos). A facade `OpenMtrClient`
delega para o adapter configurado via `System`.

```rust
pub(crate) trait MtrAdapter: Send + Sync {
    fn system(&self) -> System;
    fn nome(&self) -> &'static str;

    async fn autenticar_token(&self, ctx: &MtrCtx) -> Result<String, OpenMTRError>;
    async fn autenticar_credenciais(&self, ctx: &MtrCtx, ...) -> Result<String, OpenMTRError>;

    async fn consultar_mtrs(&self, ctx: &MtrCtx, filtro: &FiltroMtr) -> Result<LoteResult<Mtr>, OpenMTRError>;
    async fn criar_mtrs(&self, ctx: &MtrCtx, mtrs: Vec<CriarMtr>) -> Result<LoteResult<MtrCriado>, OpenMTRError>;
    // ... +14 métodos
}
```

## Envelope de Resposta

- Sucesso: `Ok(Resposta { ok: true, dados: [...], erros: [] })`
- Erro negócio: `Ok(Resposta { ok: true, dados: [...], erros: [Problema] })` (erros parciais em lote)
- Erro catastrófico: `Err(OpenMTRError { type, title, status, detail, instance })`

## Testes

```bash
cargo test -p open-mtr-core                        # 96 testes
cargo test -p open-mtr-core --test live_sinir_test  # testes contra API SINIR real
```

Tipos de teste:
- **Unitários**: validação de newtypes (Cnpj, Cpf, DataMtr), construção de Resposta
- **Wiremock**: mock da API SINIR para cenários de sucesso/erro
- **Live**: contra API real (requer `.env` com `OPENMTR_SINIR_TOKEN`)

## Novos Adapters

Para adicionar um novo sistema (ex: MTR-PE):

1. Criar `adapters/mtr_pe.rs` implementando `MtrAdapter`
2. Adicionar variante em `System` enum (`system.rs`)
3. Mapear no método `System::adapter()`
4. Opcional: testes wiremock específicos
