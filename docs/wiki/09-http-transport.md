# HttpTransport — Abstração de Transporte HTTP

O coração da refatoração openMTR. Um trait que desacopla o cliente HTTP
da implementação concreta, permitindo múltiplos backends: `reqwest` (native),
`gloo-net` (WASM browser), `globalThis.fetch` (Deno/Bun/Workers), mock para
testes.

## Trait

```rust
// core/src/openmtr/transport/mod.rs
#[async_trait]
pub trait HttpTransport: Send + Sync {
    async fn send(&self, req: HttpRequest) -> Result<HttpResponse, OpenMTRError>;
}
```

### Tipos de dados

| Tipo | Descrição |
|------|-----------|
| `HttpRequest` | `method: String`, `url: String`, `headers: Vec<(String, String)>`, `body: Option<Vec<u8>>` |
| `HttpResponse` | `status: u16`, `body: Vec<u8>`, `content_type: Option<String>`, `content_length: Option<u64>` |

### Por que um trait

A arquitetura anterior acoplava diretamente `reqwest::Client` em `MtrHttpClient`:

```
MtrHttpClient —─── reqwest::Client
TokenManager.refresh() recebia &reqwest::Client
```

Isso impedia targets WASM (sem `reqwest`), Deno (sem `tokio`), e testes
mockados. A abstração resolve todos os três.

### Implementações existentes

| Transport | Arquivo | Feature | Target |
|-----------|---------|---------|--------|
| `ReqwestTransport` | `transport/reqwest.rs` | `native-http` (default) | Linux, macOS, Windows |
| `FetchTransport` | `transport/fetch.rs` | `wasm-http` | Browser WASM (`gloo-net`) |
| `JsFetchTransport` | `wasm/src/transport.rs` | externo ao core | Deno, Bun, Workers (`globalThis.fetch`) |

## MtrHttpClient

```rust
// core/src/openmtr/client/http.rs
pub struct MtrHttpClient {
    transport: Arc<dyn HttpTransport>,  // type-erased, plug qualquer transporte
    config: Config,
    token_mgr: Arc<RwLock<TokenManager>>,
}
```

O `MtrHttpClient` é **não-genérico**: armazena o transporte como `Arc<dyn HttpTransport>`.
Isso permite que qualquer código que use `MtrHttpClient` aceite qualquer transporte
sem precisar de parâmetros de tipo.

### Responsabilidades do MtrHttpClient

| Camada | O que faz |
|--------|-----------|
| Auth | Injeta `Authorization: Bearer {token}` em toda requisição |
| Retry 401 | Ao receber 401, força refresh do token e retenta uma vez |
| Retry 5xx/429 | Backoff exponencial: 1s → 2s → 4s (3 tentativas) |
| Retry timeout | Timeout de rede também aciona retry com backoff |
| Validação | Mapeia status HTTP → `OpenMTRError` |

### Fluxo de requisição

```
adapter.consultar_mtrs(filtro)
  → ctx.http.post("/listaManifesto", body, "consultar")
    → MtrHttpClient::request("POST", ...)
      ① get_token()       — verifica cache JWT
      ② transport.send()  — delega para o transporte concreto
      ③ se 401 → refresh_token() → transport.send() (retry)
      ④ se 5xx/429/timeout → sleep(backoff) → retry
      ⑤ parse json → retorna ao adapter
```

## TokenManager

```rust
// core/src/openmtr/client/token.rs
pub struct TokenManager {
    pub integration_token: String,
    cached: Option<CachedToken>,
    pub refresh_url: Option<String>,
}
```

Desacoplado do transporte — não recebe mais `&reqwest::Client` no refresh.
A lógica de HTTP do refresh foi movida para `MtrHttpClient::refresh_token()`:

```rust
// MtrHttpClient
pub async fn refresh_token(&self) -> Result<(), OpenMTRError> {
    let (url, integration_token) = {
        let mgr = self.token_mgr.read().unwrap();
        mgr.force_refresh_data().ok_or(...)?
    };
    let req = HttpRequest::post(&url)
        .header("Authorization", format!("Bearer {integration_token}"));
    let resp = self.transport.send(req).await?;
    // parse response → set_cached_token()
}
```

### Métodos chave

| Método | Função |
|--------|--------|
| `token()` | Retorna token cached se ainda válido |
| `force_refresh_data()` | Retorna `(refresh_url, integration_token)` sempre |
| `set_cached_token(t)` | Armazena token com TTL extraído do JWT |
| `extract_expiry(jwt)` | Extrai `exp` do payload JWT |

## Feature Flags

```toml
# core/Cargo.toml
[features]
default = ["native-http"]
native-http = ["dep:reqwest", "dep:tokio", "reqwest/rustls-tls"]
wasm-http = ["dep:gloo-net", "dep:wasm-bindgen-futures", "dep:web-sys", "dep:js-sys"]
```

No target `wasm32-unknown-unknown`, `reqwest` e `tokio` **não são compilados**.
O core fica com ~0 dependências de rede — todo transporte é injetado.

## Compatibilidade WASM

Para compilar em `wasm32-unknown-unknown`, o core usa polyfills condicionais:

| std::time | WASM polyfill |
|-----------|---------------|
| `SystemTime::now()` | `js_sys::Date::now()` via `now_unix_secs()` |
| `Instant::now()` | `web_time::Instant` |
| `Utc::now()` | `time_util::utc_now()` → `js_sys::Date` |
| `Uuid::now_v7()` | uuid `js` feature → `Date.now()` |

### sleep() para retry

```rust
#[cfg(feature = "native-http")]
async fn sleep(d: Duration) { tokio::time::sleep(d).await; }

#[cfg(all(not(feature = "native-http"), target_arch = "wasm32"))]
async fn sleep(_d: Duration) { /* no-op: WASM single-threaded */ }

#[cfg(all(not(feature = "native-http"), not(target_arch = "wasm32")))]
async fn sleep(d: Duration) { std::thread::sleep(d); }
```

## Criando um Transporte Novo

Qualquer crate pode implementar `HttpTransport` e injetar no builder:

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

Toda a lógica de auth, retry, backoff e validação já está no `MtrHttpClient` —
o transporte só precisa implementar `send()`.
