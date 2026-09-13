# WASM / Deno — Integração WebAssembly

openMTR funciona em Deno 2+ via WebAssembly compilado a partir do Rust,
usando `wasm-bindgen` + `globalThis.fetch()` como transporte HTTP.

## Funcionamento

```
Deno/TS
  │  import init, { OpenMtrWasm } from "./pkg/open_mtr_wasm.js"
  ▼
┌─────────────────────────────────────────┐
│  open_mtr_wasm_bg.wasm (wasm32-unknown) │
│  ┌───────────────────────────────────┐  │
│  │  #[wasm_bindgen]                  │  │
│  │  class OpenMtrWasm {              │  │
│  │    init(), consultar_mtrs(), etc. │  │
│  │    └─ core::OpenMtrClient         │  │
│  │       └─ MtrHttpClient            │  │
│  │          └─ JsFetchTransport ─────┼──┼──► globalThis.fetch()
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

O WASM importa `fetch` do escopo global JavaScript (`globalThis.fetch()`),
que em Deno é nativo — sem DOM, sem `window`, sem `web_sys`.

## JsFetchTransport

```rust
// wasm/src/transport.rs
pub struct JsFetchTransport;

#[async_trait]
impl HttpTransport for JsFetchTransport {
    async fn send(&self, req: HttpRequest) -> Result<HttpResponse, OpenMTRError> {
        // 1. Chama globalThis.fetch() via js_sys::Reflect
        // 2. Bridge Send/não-Send via oneshot::channel + spawn_local
        // 3. Converte Response JS → HttpResponse Rust
    }
}
```

### Por que spawn_local + oneshot

`JsFuture::from(promise)` é `!Send` (contém `Rc<RefCell<...>>`).
O trait `HttpTransport` exige `Send + Sync`.

A ponte:
1. **`spawn_local`**: executa a `do_fetch()` (não-Send) no event loop JS
2. **`oneshot::channel`**: o `Sender` é `Send`, o `Receiver` é `Send`
3. O `Receiver.await` é `Send` — satisfaz o trait

```rust
let (tx, rx) = futures::channel::oneshot::channel();

wasm_bindgen_futures::spawn_local(async move {
    let result = do_fetch(req).await;
    let _ = tx.send(result);           // não-Send → Send bridge
});

rx.await                             // Send future ✓
```

## Build

```bash
# 1. Compilar Rust → WASM
cargo build -p open-mtr-wasm --target wasm32-unknown-unknown --release

# 2. Gerar JS glue + TypeScript types
wasm-bindgen target/wasm32-unknown-unknown/release/open_mtr_wasm.wasm \
  --out-dir wasm/pkg --target web
```

### Artefatos gerados

```
wasm/pkg/
├── open_mtr_wasm_bg.wasm     # Binário WASM (~660 KB)
├── open_mtr_wasm_bg.wasm.d.ts # TypeScript para o binário
├── open_mtr_wasm.js           # JS glue code
├── open_mtr_wasm.d.ts         # TypeScript para a API
└── snippets/                  # JS auxiliar
```

## API Exportada

```ts
// wasm/pkg/open_mtr_wasm.d.ts (gerado automaticamente)
export class OpenMtrWasm {
    constructor();

    init(system: string, token: string, base_url: string): Promise<string>;
    health(): Promise<string>;

    consultar_mtrs(filtro_json: string): Promise<string>;
    criar_mtrs(json: string): Promise<string>;
    receber_mtrs(json: string): Promise<string>;
    cancelar_mtrs(json: string): Promise<string>;
    modificar_mtrs(json: string): Promise<string>;
    aceitar_alteracoes(json: string): Promise<string>;
    download_mtrs(json: string): Promise<string>;

    gerar_cdfs(json: string): Promise<string>;
    consultar_cdfs(json: string): Promise<string>;
    download_cdfs(json: string): Promise<string>;

    listar_residuos(): Promise<string>;
    listar_acondicionamentos(): Promise<string>;
    listar_classes(): Promise<string>;
    listar_estados_fisicos(): Promise<string>;
    listar_tratamentos(): Promise<string>;
    listar_unidades(): Promise<string>;

    listar_classes_por_residuo(codigo_ibama: string): Promise<string>;
    listar_acondicionamentos_por_estado_fisico(codigo_estado: number): Promise<string>;
    criar_manifesto_complementar(json: string): Promise<string>;
}
```

Todos os métodos retornam JSON strings (mesmo padrão do napi-rs).

## Exemplo Completo (Deno)

```ts
// deno run --allow-read --allow-net main.ts
import init, { OpenMtrWasm } from "./wasm/pkg/open_mtr_wasm.js";

// Carregar token do .env
const env = Deno.readTextFileSync("../.env");
const token = env.match(/SINIR_REFRESH_TOKEN=(.+)/)?.[1]?.trim() ?? "";

// Inicializar WASM
await init();
const client = new OpenMtrWasm();

// Autenticar
const initResult = JSON.parse(await client.init("sinir", token, ""));
console.log("init:", initResult.ok); // true

// Listar resíduos
const residuos = JSON.parse(await client.listar_residuos());
console.log(`Total: ${residuos.dados.length}`); // 895

// Consultar MTRs
const mtrs = JSON.parse(await client.consultar_mtrs(JSON.stringify({
    periodo: { dias: 7 },   // atalho: últimos 7 dias
    tamanho: 5
})));
console.log(`MTRs: ${mtrs.dados.length}`);

// Health check
const health = JSON.parse(await client.health());
console.log(health.mensagem);
```

### Formato do período no JSON

O `periodo` aceita 3 formas — o Rust resolve internamente para o formato que
cada sistema espera (ex.: `dd-MM-yyyy` no SINIR/MTR-PE):

```js
// 1. Atalho: últimos N dias até hoje
periodo: { dias: 30 }

// 2. Objeto Date do JS (new Date()) — serializado como ISO 8601
periodo: { dataInicial: new Date(Date.now() - 30*864e5), dataFinal: new Date() }

// 3. Intervalo explícito dd-MM-yyyy (ou ISO 8601 / timestamp ms)
periodo: { dataInicial: "01-06-2026", dataFinal: "10-08-2026" }
```

## Compatibilidade de Runtime

| Runtime | `globalThis.fetch` | Funciona? |
|---------|-------------------|-----------|
| Deno 2+ | ✅ Nativo | ✅ |
| Navegador | ✅ Nativo | ✅ |
| Bun | ✅ Nativo | ✅ |
| Cloudflare Workers | ✅ Nativo | ✅ |
| Node.js 18+ | ✅ (experimental) | ⚠️ |

## Polyfills WASM

O target `wasm32-unknown-unknown` não implementa relógio.
openMTR substitui todas as chamadas de tempo com equivalentes JS:

| Chamada Rust | Substituição WASM | Arquivo |
|-------------|-------------------|---------|
| `SystemTime::now()` | `js_sys::Date::now()` | `client/http.rs → now_unix_secs()` |
| `Instant::now()` | `web_time::Instant` | `client/http.rs`, `client/token.rs` |
| `Utc::now()` | `time_util::utc_now()` | `time_util.rs` |
| `Uuid::now_v7()` | uuid `js` feature | `Cargo.toml` |
| `chrono::Utc::now()` | `wasmbind` feature + `utc_now()` | `Cargo.toml`, `time_util.rs` |

## Dependências WASM (core)

```toml
# Compiladas apenas para wasm32
[target.'cfg(target_arch = "wasm32")'.dependencies]
web-time = "1"                                    # Instant::now()
chrono = { version = "0.4", features = ["serde", "wasmbind", "std"] }
uuid = { version = "1", features = ["std", "v7", "serde", "js"] }
js-sys = "0.3"                                    # Date::now()
wasm-bindgen-futures = "0.4"                      # JsFuture, spawn_local
```

## Limitações

| Aspecto | Limitação |
|---------|-----------|
| Retry sleep | No-op no WASM (single-threaded — sem `tokio::time::sleep`) |
| Timeout HTTP | Implementado via `Promise.race` no JS (não via `reqwest::timeout`) |
| Memória | Binário ~660 KB (release, stripped, LTO) |
| Threads | Single-threaded — `Send` bounds são satisfeitos via pontes `unsafe` |
