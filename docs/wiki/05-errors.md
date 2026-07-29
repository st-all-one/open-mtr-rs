# Error System

RFC 7807 (Problem Details) com UUID v7 para correlação distribuída.

## MtrError

```rust
pub enum MtrError {
    Api { category, status, detail, instance, context },
    Http(reqwest::Error),
    RateLimited { retry_after, context },
    Timeout { duration, context },
    Serialization(serde_json::Error),
    Config(String),
}
```

## ErrorCategory

```rust
pub enum ErrorCategory {
    AuthenticationFailed,   // 401
    AuthorizationDenied,    // 403
    ResourceNotFound,       // 404
    ValidationError,        // 422
    Conflict,               // 409
    RateLimited,            // 429
    Timeout,                // 504
    NetworkError,           // 503
    ParseError,             // 500
    InternalError,          // 500
}
```

## ErrorContext

```rust
pub struct ErrorContext {
    pub operation: Option<String>,
    pub http_status: Option<u16>,
    pub api_errors: Option<Vec<String>>,
    pub response_body: Option<String>,
}
```

## Construtores

| Método | Uso |
|--------|-----|
| `MtrError::api(cat, status, detail, ctx)` | Erro da API |
| `MtrError::validation(msg)` | Erro de validação |
| `MtrError::config(msg)` | Erro de configuração |
| `MtrError::deserialize(msg)` | Erro de parser |
| `MtrError::timeout(duration, op)` | Timeout |

## Utilitários

```rust
err.category()      // → Option<ErrorCategory>
err.is_retryable()  // → bool (429, 5xx, timeout)
```

## Exemplo

```json
{
    "category": "validation-error",
    "status": 422,
    "detail": "Justificativa deve ter no mínimo 20 caracteres",
    "instance": "0194b3e0-7f1a-7d80-8000-123456789abc",
    "context": { "operation": "manifesto.cancelar" }
}
```
