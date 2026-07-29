# Cliente HTTP

O `MtrHttpClient` é o wrapper genérico sobre `reqwest` usado internamente pelos adapters.

## Características

- **Retry**: 3 tentativas com backoff exponencial (1s, 2s, 4s)
- **Refresh JWT**: Renovação automática em 401 (1 retry)
- **Tracing**: Todos os métodos instrumentados com `#[instrument]`
- **Timeout**: Configurável via `Config.timeout`
- **Rate limiting**: Sliding window configurável via `Config.max_rps`

## Métodos

```rust
impl MtrHttpClient {
    // GET com parse JSON
    async fn get<T: DeserializeOwned>(path, op) -> Result<T>;

    // POST com body JSON
    async fn post<T, B>(path, body, op) -> Result<T>;

    // POST que retorna bytes (download)
    async fn post_bytes(path, op) -> Result<Vec<u8>>;

    // GET que retorna bytes
    async fn get_bytes(path, op) -> Result<Vec<u8>>;

    // Genérico
    async fn request<T, B>(method, path, body, op) -> Result<T>;
    async fn request_raw(method, path, op) -> Result<Vec<u8>>;
}
```

## TokenManager

Gerencia o ciclo de vida do JWT:

- Cache em memória com expiry tracking
- Refresh automático via POST para URL configurada
- Fallback de 8h se JWT não tiver claim `exp`
- Margem de segurança de 60s antes do expiry real

## Uso no adapter

```rust
async fn minhas_consultas(&self, ctx: &MtrCtx) -> Result<..., OpenMTRError> {
    // POST
    let resp: Value = ctx.http.post("/api/endpoint", &body, "op").await?;

    // GET
    let resp: Value = ctx.http.get("/api/endpoint", "op").await?;

    // Download
    let bytes = ctx.http.post_bytes("/api/download", "op").await?;
}
```
