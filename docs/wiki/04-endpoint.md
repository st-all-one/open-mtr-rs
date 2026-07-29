# trait Endpoint + PaginatedIter

## trait Endpoint

```rust
pub trait Endpoint {
    type Response: DeserializeOwned;
    fn method(&self) -> Method;
    fn path(&self) -> Cow<'static, str>;
    fn query_params(&self) -> Vec<(String, String)> { vec![] }
    fn body(&self) -> Option<Value> { None }
}
```

Implementações concretas:

| Struct | Response | Method | Path |
|--------|----------|--------|------|
| `ListarManifestos { dto }` | `ObjetoRespostaRest<Vec<ManifestoResumo>>` | POST | `/listaManifesto` |
| `DownloadPdf { man_numero }` | `Vec<u8>` | POST | `/downloadManifesto/{num}` |
| `CancelarManifesto { dto }` | `ObjetoRespostaRest<CancelamentoResponse>` | POST | `/cancelarManifesto` |

## MtrClient::call

```rust
pub async fn call<E: Endpoint>(&self, ep: E) -> Result<E::Response, MtrError>
```

Roteia automaticamente:
- `body()` com `Some` → `request_with_body`
- `query_params()` não vazio → `request_with_params`
- Caso contrário → `request`

Para downloads:

```rust
pub async fn call_download<E: Endpoint<Response = Vec<u8>>>(&self, ep: E)
    -> Result<Vec<u8>, MtrError>
```

## PaginatedIter

```rust
pub struct PaginatedIter {
    client: MtrClient,
    filtro_base: ApiwsListaManifestoDTO,
    page: i32,
    page_size: i32,
    total_elements: i64,
    buffer: Vec<ManifestoResumo>,
    idx: usize,
    done: bool,
}
```

```rust
let mut iter = PaginatedIter::new(client, filtro_base, 100);

// Pull mode
while let Some(item) = iter.next().await? {
    process(item);
}

// Collect all
let todos: Vec<ManifestoResumo> = iter.collect_all().await?;
```

Lazy: só busca próxima página quando o buffer atual acaba.
