# openMTR — Guia do Agente

## TL;DR

```bash
cargo build -p open-mtr-core
cargo test -p open-mtr-core                            # 79 unitários
cargo test -p open-mtr-core --test live_sinir_test      # 14 live SINIR
cargo clippy -p open-mtr-core -- -D warnings            # 0 warnings

# WASM (Deno)
cargo build -p open-mtr-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/open_mtr_wasm.wasm \
  --out-dir wasm/pkg --target web
deno run --allow-read --allow-net wasm/examples/main.ts
```

## Stack

| Componente | Tecnologia |
|---|---|
| Runtime | Rust 1.99+ (edition 2024) |
| Async (native) | `tokio` (sync, time) — **opcional**, feature `native-http` |
| HTTP (native) | `reqwest` 0.12 + `rustls` — **opcional**, feature `native-http` |
| HTTP (WASM) | `globalThis.fetch()` via `js_sys` — sem `reqwest`, sem `tokio` |
| Transport abstraction | Trait `HttpTransport` — `Arc<dyn HttpTransport>` type-erased |
| Polyfills WASM | `web_time` (Instant), `js_sys::Date` (SystemTime), `time_util::utc_now()` |
| Serialização | `serde` + `serde_json` |
| Erros | RFC 9457 (Problem Details) + UUID v7 |
| Logs | `tracing` — spans, `#[instrument]`, structured |
| Retry | Backoff exponencial (3 tries, 1s/2s/4s) |
| Bindings | napi-rs (Node.js), UniFFI (Swift/Kotlin/Python), wasm-bindgen (Deno/Web) |

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
│   ├── adapters/             # Implementações específicas (cada sistema é um adapter)
│   │   ├── mod.rs            # trait MtrAdapter, MtrCtx, ItemResult, LoteResult
│   │   ├── sinir.rs          # SinirAdapter (~1117 linhas)
│   │   ├── mtr_pe.rs         # MtrPeAdapter (~799 linhas)
│   │   └── sigor.rs          # SigorAdapter (~862 linhas)
│   ├── transport/            # ★ NOVO: abstração de transporte HTTP
│   │   ├── mod.rs            # trait HttpTransport, HttpRequest, HttpResponse
│   │   ├── reqwest.rs        # ReqwestTransport (feature native-http)
│   │   └── fetch.rs          # FetchTransport (feature wasm-http)
│   ├── client/               # HTTP genérico
│   │   ├── http.rs           # MtrHttpClient (retry, auth, rate-limit)
│   │   └── token.rs          # TokenManager (refresh JWT, sem lógica HTTP)
│   ├── config.rs             # Config, ConfigBuilder
│   ├── error.rs              # OpenMTRError (RFC 9457)
│   ├── resposta.rs           # Resposta<T>, Problema
│   ├── system.rs             # enum System + adapter()
│   ├── time_util.rs          # ★ NOVO: utc_now() polyfill WASM
│   ├── open_mtr_client.rs    # OpenMtrClient + Builder (21 métodos)
│   ├── open_mtr_auth.rs      # OpenMtrClassicAuth (CPF+senha → token) + AuthParams
│   └── prelude.rs            # re-exports públicos

open-mtr-napi (cdylib)         # Node.js N-API
└── src/lib.rs                # #[napi] OpenMtrClient

open-mtr-uniffi (cdylib)       # Swift/Kotlin/Python
└── src/lib.rs                # UniFFI exports

open-mtr-wasm (cdylib)         # ★ NOVO: Deno/Web WASM
├── src/lib.rs                # #[wasm_bindgen] OpenMtrWasm
├── src/transport.rs          # JsFetchTransport (globalThis.fetch)
└── examples/main.ts          # Demo SINIR funcional
```

## Convenções de código

- `snake_case` para funções e variáveis; `CamelCase` para tipos
- Todos os tipos de domínio: `Debug, Clone, Serialize, Deserialize`
- Campos opcionais: `Option<T>` com `#[serde(skip_serializing_if = "Option::is_none")]`
- Métodos públicos retornam `MtrResult<T> = Result<Resposta<T>, OpenMTRError>`
- Todo método tem `tracing::info_span!` com `instance` (UUID v7), `operation`, `system`
- `unwrap()` proibido em produção (use `?` ou `expect()` com mensagem)
- `#[non_exhaustive]` em enums públicos
- Polyfills WASM: `#[cfg(target_arch = "wasm32")]` em vez de feature flags para tempo

## HttpTransport (abstração de rede)

```rust
// core/src/openmtr/transport/mod.rs
#[async_trait]
pub trait HttpTransport: Send + Sync {
    async fn send(&self, req: HttpRequest) -> Result<HttpResponse, OpenMTRError>;
}

// MtrHttpClient usa Arc<dyn HttpTransport> — type-erased, plug qualquer transporte
pub struct MtrHttpClient {
    transport: Arc<dyn HttpTransport>,
    config: Config,
    token_mgr: Arc<RwLock<TokenManager>>,
}
```

### Adicionar novo transporte

```rust
struct MeuTransport;

#[async_trait]
impl HttpTransport for MeuTransport {
    async fn send(&self, req: HttpRequest) -> Result<HttpResponse, OpenMTRError> {
        // ~30 linhas — enviar requisição, retornar resposta
    }
}

let client = OpenMtrClient::builder()
    .base(System::Sinir)
    .token("jwt...")
    .build_with_transport(MeuTransport)
    .await?;
```

Toda a lógica de auth, retry, backoff, validação já está no `MtrHttpClient`.

## Adapter Pattern

Cada sistema implementa `trait MtrAdapter` (18 métodos + 4 SIGOR-specific com default).
A facade `OpenMtrClient` delega para o adapter configurado via `System`.

```rust
pub trait MtrAdapter: Send + Sync {
    fn system(&self) -> System;
    fn nome(&self) -> &'static str;

    async fn autenticar_token(&self, ctx: &MtrCtx) -> Result<String, OpenMTRError>;
    async fn autenticar_credenciais(&self, ctx: &MtrCtx, params: &AuthParams) -> Result<String, OpenMTRError>;

    async fn consultar_mtrs(&self, ctx: &MtrCtx, filtro: &FiltroMtr) -> Result<LoteResult<Mtr>, OpenMTRError>;
    async fn criar_mtrs(&self, ctx: &MtrCtx, mtrs: Vec<CriarMtr>) -> Result<LoteResult<MtrCriado>, OpenMTRError>;
    // ... +14 métodos universais
    // ... +4 métodos SIGOR-specific com default Err(not_supported)
}
```

## Envelope de Resposta

- Sucesso: `Ok(Resposta { ok: true, dados: [...], erros: [] })`
- Erro negócio: `Ok(Resposta { ok: true, dados: [...], erros: [Problema] })` (erros parciais em lote)
- Erro catastrófico: `Err(OpenMTRError { type, title, status, detail, instance })`

## Testes

```bash
cargo test -p open-mtr-core                              # 79 unitários
cargo test -p open-mtr-core --test live_sinir_test        # 14 live SINIR
cargo test -p open-mtr-core --test domain_test            # 8 domain
cargo test -p open-mtr-core --test transport_integration  # mock transport
```

Tipos de teste:
- **Unitários**: validação de newtypes (Cnpj, Cpf, DataMtr), construção de Resposta, mock HttpTransport
- **Wiremock**: mock da API SINIR para cenários de sucesso/erro
- **Live**: contra API real (requer `.env` com `OPENMTR_SINIR_TOKEN`)
- **Token**: cache, refresh, force_refresh, extract_expiry

## Novos Adapters

Para adicionar um novo sistema (ex: MTR-CE):

1. Criar `adapters/mtr_ce.rs` implementando `MtrAdapter`
2. Adicionar variante em `System` enum (`system.rs`)
3. Mapear no método `System::adapter()`
4. Opcional: testes wiremock específicos

## WASM Build

```bash
# Compilar
cargo build -p open-mtr-wasm --target wasm32-unknown-unknown --release

# Gerar JS glue + TypeScript
wasm-bindgen target/wasm32-unknown-unknown/release/open_mtr_wasm.wasm \
  --out-dir wasm/pkg --target web

# Testar com Deno
cd wasm && deno run --allow-read --allow-net examples/main.ts
```

Ver [docs/wiki/11-wasm-deno.md](wiki/11-wasm-deno.md) para detalhes completos.
