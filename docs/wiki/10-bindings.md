# Bindings — Targets de Distribuição

openMTR expõe sua API em **4 targets** através de crates de binding dedicados.
Todos compartilham o mesmo `core` e a mesma abstração `HttpTransport`.

## Tabela de Targets

| Target | Crate | Transporte | Async Runtime | Formato |
|--------|-------|------------|---------------|---------|
| Rust native | `open-mtr-core` | `ReqwestTransport` | `tokio` | Tipos Rust nativos |
| Node.js (NAPI) | `open-mtr-napi` | `ReqwestTransport` | `tokio` (napi crate) | JSON strings |
| Mobile/Desktop (FFI) | `open-mtr-uniffi` | `ReqwestTransport` | `tokio` (global, block_on) | JSON strings |
| Deno/Browser (WASM) | `open-mtr-wasm` | `JsFetchTransport` | JS event loop | JSON strings |

## napi-rs (Node.js)

```toml
# napi/Cargo.toml
[dependencies]
open-mtr-core = { path = "../core" }
napi = { version = "2", features = ["async", "serde-json", "napi8"] }
napi-derive = "2"
```

### Exportações

```rust
#[napi]
pub struct OpenMtrClient { /* wraps core::OpenMtrClient via OnceCell */ }

#[napi]
impl OpenMtrClient {
    #[napi(constructor)]
    pub fn new() -> Self { ... }

    #[napi]
    pub async fn init(&self, system: String, token: String, base_url: String) -> napi::Result<()> { ... }

    #[napi]
    pub async fn consultar_mtrs(&self, filtro_json: String) -> napi::Result<RespostaJson> { ... }

    // + 16 métodos adicionais
}
```

### Async Runtime

O `napi` crate gerencia o runtime tokio internamente.
O `init()` builda o `OpenMtrClient` via `OnceCell::get_or_try_init()`.
Todas as chamadas são `async` — Node.js integra via libuv.

### Uso (TypeScript)

```ts
import { OpenMtrClient } from "open-mtr-napi";

const client = new OpenMtrClient();
await client.init("sinir", token, "");
const result = client.consultarMtrs(JSON.stringify(filtro));
```

## UniFFI (Mobile/Desktop)

```toml
# uniffi/Cargo.toml
[dependencies]
open-mtr-core = { path = "../core" }
uniffi = "0.28"
```

### Exportações

Interface definida via UDL (`uniffi/src/open_mtr.udl`):

```
interface OpenMtrClient {
    [Throws=OpenMtrError]
    string consultar_mtrs(string filtro_json);

    // + 16 métodos
};
```

### Async Runtime

Tokio runtime **global** (`OnceLock<Runtime>`) — criado na primeira chamada.
Todas as operações usam `runtime.block_on()` — **bloqueiam a thread chamadora**.
Isso é aceitável para FFI síncrono (Swift, Kotlin, Python).

### Mapeamento de Erros

`OpenMTRError` é colapsado em enum flat para a FFI:

```
enum OpenMtrError {
    Api,
    Network,
    Timeout,
    Auth,
    Config,
    Validation,
};
```

## WASM (Deno/Browser)

Ver documentação completa em [11-wasm-deno.md](11-wasm-deno.md).

```toml
# wasm/Cargo.toml
[dependencies]
open-mtr-core = { path = "../core", default-features = false }
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
```

### Bridge Send/não-Send

`JsFuture` não implementa `Send`, mas o trait `HttpTransport` requer `Send + Sync`.
A solução usa `oneshot::channel` + `spawn_local` como ponte:

```rust
#[async_trait]
impl HttpTransport for JsFetchTransport {
    async fn send(&self, req: HttpRequest) -> Result<HttpResponse, OpenMTRError> {
        let (tx, rx) = futures::channel::oneshot::channel();

        wasm_bindgen_futures::spawn_local(async move {
            let result = do_fetch(req).await;
            let _ = tx.send(result);
        });

        rx.await.map_err(|_| OpenMTRError::network("fetch task cancelled"))?
    }
}
```

### Build Pipeline

```bash
cargo build -p open-mtr-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/open_mtr_wasm.wasm \
  --out-dir wasm/pkg --target web
```

### Uso (Deno)

```ts
import init, { OpenMtrWasm } from "./pkg/open_mtr_wasm.js";

await init();
const client = new OpenMtrWasm();
await client.init("sinir", token, "");
const result = JSON.parse(await client.listar_residuos());
// result.dados.length === 895
```
