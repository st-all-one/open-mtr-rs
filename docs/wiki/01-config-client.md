# Configuração + Client + Token

## MtrConfig

Construído via **Builder pattern** — imutável após construção.

```rust
let config = MtrConfig::builder()
    .base_url("https://admin.sinir.gov.br/api")
    .integration_token(std::env::var("MTR_INTEGRATION_TOKEN")?)
    .timeout_secs(60)
    .max_rps(10)
    .build()?;
```

Campos validados em `build()`:
- `base_url` — obrigatória, trailing slash removido
- `integration_token` — obrigatório
- `auth_method` — default `BearerToken` (único suportado)
- `timeout` — default 60s
- `max_rps` — default 10

## MtrClient

```rust
#[derive(Clone)]
pub struct MtrClient {
    http: Client,
    config: MtrConfig,
    token_mgr: Arc<RwLock<TokenManager>>,
}
```

### Métodos públicos de request

| Método | Uso |
|--------|-----|
| `request::<T>()` | GET sem body |
| `request_with_body::<T, B>()` | POST/PUT com body |
| `request_with_params::<T>()` | GET com query params |
| `download_raw()` | Retorna `Vec<u8>` (PDF/XML) |

### Identificador de operação

Toda requisição recebe uma string `op` para tracing:

```rust
self.client.request_with_body(..., "manifesto.listar").await
```

### Auto-renovação de token

1. `get_token()` — retorna token cacheado se válido
2. Se expirado ou ausente → `refresh_token()` → POST `/token`
3. Se 401 → refresh automático + retry 1x

## TokenManager

```rust
pub struct TokenManager {
    integration_token: String,
    cached: Option<CachedToken>,
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,  // now + expiraEm - 60s de margem
}
```

- `token()` → `Option<&str>` — retorna só se `Instant::now() < expires_at`
- `refresh()` → POST `/token` com `Bearer <integration_token>`

## Rate Limiting

Sliding window via `Mutex<SlidingWindow>` no client interno:

```rust
pub struct SlidingWindow {
    max_requests: u32,     // default 10
    window: Duration,      // 1 segundo
    timestamps: VecDeque<Instant>,
}
```

## Retry

```rust
pub async fn with_retry<T, F, Fut>(f: F) -> Result<T, MtrError>
```

- Máx 3 tentativas
- Backoff exponencial: 1s, 2s, 4s
- Só retenta se `e.is_retryable()` (429, 5xx, timeout)
