# mtr-sinir

Wrapper Rust para a API SINIR (Sistema Nacional de Informações sobre a Gestão dos Resíduos Sólidos).

## Stack

| Camada | Tecnologia |
|--------|-----------|
| Runtime | Rust 1.85+ (edition 2024), `tokio` async |
| HTTP | `reqwest` 0.12 (rustls-tls) |
| Serialização | `serde` + `serde_json` |
| Erros | `thiserror` — RFC 7807 + UUID v7 |
| Logs | `tracing` — spans, `#[instrument]` |
| Cache | `Arc<RwLock>` ou `moka` (opt) |
| Rate limit | Sliding window |
| Retry | Backoff exponencial (3 tries) |

## Arquitetura

```
mtr-sinir/
├── core/          # Crate raiz (100% agnóstico)
│   ├── src/
│   │   ├── client/     # MtrClient, TokenManager, retry, trait Endpoint
│   │   ├── core/       # MtrConfig, MtrError (RFC 7807)
│   │   ├── schemas/    # 10 domínios (manifesto, recebimento, CDF, LR, IE, ...)
│   │   └── services/   # Lógica de negócio + builders typestate
│   └── tests/          # Testes de integração (wiremock)
├── napi/          # Bindings napi-rs → Node.js / TypeScript / Bun
│   └── src/lib.rs
├── uniffi/        # Bindings UniFFI → Python / Kotlin / Swift / Ruby / Go / C#
│   ├── src/
│   │   ├── lib.rs
│   │   └── mtr_sinir.udl
│   └── bindings/      # (gerado)
└── docs/
    ├── wiki/          # Documentação interna
    └── AGENTS.md      # Guia do agente AI
```

## Começando

```bash
# Rust toolchain
rustup toolchain install 1.85.0
rustup default 1.85.0

# Build + test
cargo build
cargo test                    # 42 tests
cargo clippy                  # 0 warnings

# Documentação
cargo doc --open -p mtr-sinir-core
```

## Uso (Rust puro)

```rust
use mtr_sinir_core::client::http::MtrClient;
use mtr_sinir_core::core::config::MtrConfig;
use mtr_sinir_core::services::manifesto::ManifestoService;
use mtr_sinir_core::schemas::common::DataMtr;
use mtr_sinir_core::schemas::manifesto::TipoParceiro;

let client = MtrClient::new(
    MtrConfig::builder()
        .base_url("https://admin.sinir.gov.br/api")
        .integration_token(std::env::var("MTR_INTEGRATION_TOKEN")?)
        .build()?,
)?;

let service = ManifestoService::new(client);
let (lista, total) = service
    .listar(
        DataMtr::parse("01-01-2024")?,
        DataMtr::parse("31-01-2024")?,
        TipoParceiro::Gerador,
        0, 10,
    )
    .await?;
```

## Workspace Members

### core (`mtr-sinir-core`)

Crate principal com toda a lógica de negócio, schemas, e integração HTTP.

**Endpoints suportados:**
- Auth: POST `/token`
- Manifesto: listar, cancelar, download PDF
- Recebimento: registrar lote, alterar, aceitar
- CDF: emitir, emitir LR, download
- Logística Reversa: origem, processo, alterar titularidade
- Integração Estadual: integrar manifestos, integrar recebimento
- Tabelas: acondicionamento, unidades, classes, estados físicos

### napi (`mtr-sinir-napi`)

Bindings nativos para Node.js/TypeScript via [napi-rs](https://napi.rs).

```bash
cd napi && npm ci && npm run build
```

```typescript
import { MtrSinirClient } from '@mtr-sinir/native';

const client = new MtrSinirClient(
  process.env.MTR_INTEGRATION_TOKEN!,
  'https://admin.sinir.gov.br/api',
);

const lista = await client.listarManifestos('01-01-2024', '31-01-2024', 8, 0, 10);
const pdf = await client.downloadPdf('00001');
```

### uniffi (`mtr-sinir-uniffi`)

Bindings para Python / Kotlin / Swift / Ruby / Go / C# via [UniFFI](https://mozilla.github.io/uniffi-rs/).

```bash
cargo build -p mtr-sinir-uniffi --release
uniffi-bindgen generate uniffi/src/mtr_sinir.udl --language python
```

```python
from mtr_sinir_uniffi import MtrSinirClient

client = MtrSinirClient.init("token", "https://admin.sinir.gov.br/api")
lista = client.listar_manifestos("01-01-2024", "31-01-2024", 8, 0, 10)
```

## Domain Newtypes

Tipos com validação em Rust para segurança de dados:

| Tipo | Validação | Serialização |
|------|-----------|-------------|
| `Cnpj` | 14 dígitos + check digits | string limpa ou formatada |
| `Cpf` | 11 dígitos + check digits | string limpa ou formatada |
| `CpfCnpj` | 11→CPF, 14→CNPJ | automática |
| `DataMtr` | `dd-MM-yyyy` | string |
| `Pagina` | ≥ 1 | i32 |
| `TamanhoPagina` | 1..=50 | i32 |
| `TimestampMs` | ms desde epoch | i64 |

## Builders com Typestate

Padrão builder que usa o sistema de tipos do Rust para garantir chamadas corretas em tempo de compilação:

- `ManifestoListBuilder<StFaltando → StPronto>` — `periodo()` obrigatório antes de `build()`
- `SalvarManifestoLoteBuilder<SemResiduos → ComResiduos>` — `add_residuo()` obrigatório antes de `build()`

## trait Endpoint

Abstração type-safe sobre endpoints HTTP:

```rust
pub trait Endpoint {
    type Response: DeserializeOwned;
    fn method(&self) -> Method;
    fn path(&self) -> Cow<'static, str>;
}
```

Implementações: `ListarManifestos`, `DownloadPdf`, `CancelarManifesto`.

## Testes

```bash
cargo test                           # 42 testes
cargo test --test integration_test   # só wiremock
```

- Testes unitários de schema: inline `#[cfg(test)]` nos source files
- Testes de integração: `wiremock` simulando API SINIR

## Docker

```bash
docker build -t mtr-sinir .
docker run --rm -it mtr-sinir
```

## Licença

MPL-2.0
