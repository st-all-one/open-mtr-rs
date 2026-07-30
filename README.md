<div align="center">

# openMTR

**Shell unificado para gestão de MTRs — assíncrono, multi-sistema, tipos universais**

[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-2b3a42?style=for-the-badge)](https://opensource.org/licenses/MPL-2.0)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Made in Brazil](https://img.shields.io/badge/Made_in-Brazil-009739?style=for-the-badge)](https://github.com/topics/brazil)

</div>

**openMTR** é um wrapper multi-sistema para APIs de Manifesto de Transporte de Resíduos (MTR). Ele unifica diferentes implementações estaduais e nacional (SINIR) sob uma mesma interface, com tipos universais de domínio, envelope de resposta padronizado (RFC 9457) e zero acoplamento a sistemas específicos.

---

## 📖 Índice

- [Arquitetura](#-arquitetura)
- [Quick-start](#-quick-start)
- [Sistemas Suportados](#-sistemas-suportados)
- [16 Métodos Universais](#-16-métodos-universais)
- [Adapter Pattern](#-adapter-pattern)
- [Envelope de Resposta](#-envelope-de-resposta)
- [Enriquecimento Detalhado](#-enriquecimento-detalhado)
- [Uso TypeScript / Python](#-uso-typescript--python)
- [Testes](#-testes)
- [Stack](#-stack)
- [Comparação com Wrappers Específicos](#-comparação-com-wrappers-específicos)

---

## 🏗️ Arquitetura

```
openMTR
└── OpenMtrClient (facade única — 17 métodos universais)
    └── MtrAdapter (trait interna — 1 implementação por sistema)
        ├── SinirAdapter    → SINIR Nacional  (✅ completo)
        ├── MtrPeAdapter    → MTR Pernambuco  (✅ completo)
        ├── SigorAdapter    → SIGOR / CETESB-SP (✅ completo)
        └── ...             → +5 sistemas      (🚧 pendente)
```

### Camadas

| Camada | Responsabilidade |
|---|---|
| **OpenMtrClient** | Fachada pública — 16 métodos agnósticos que o usuário chama |
| **MtrAdapter** | Contrato interno — cada sistema implementa à sua maneira |
| **domain/** | Tipos universais (`Mtr`, `Parte`, `Residuo`, `FiltroMtr`, `Periodo`, etc.) |
| **client/** | HTTP genérico (`reqwest`, retry 3×, refresh JWT, rate-limit) |
| **error.rs** | `OpenMTRError` (catastrófico, RFC 9457) |
| **resposta.rs** | `Resposta<T>` (envelope único para sucesso e erro de negócio) |

```
┌── Entrada do usuário ────────────────────────────────────────────┐
│                                                                   │
│  mtr.consultar_mtrs(&FiltroMtr {                                  │
│      periodo: Some(Periodo::ultimos_dias(30)),                    │
│      parte: Some(FiltroParte { tipo: TipoParte::Transportador }), │
│      detalhado: true,                                             │
│      ..Default::default()                                         │
│  }).await?;                                                       │
│                                                                   │
└──────────────────────────┬────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────── SinirAdapter ──────────────────────────────────┐
│                                                                   │
│  1. POST /listaManifesto → extrai números                         │
│  2. GET  /retornaManifesto/{n}  ← paralelo (N requisições)        │
│  3. Monta Vec<Mtr> com resíduos, CNPJs, campos completos          │
│                                                                   │
└──────────────────────────┬────────────────────────────────────────┘
                           │
                           ▼
┌────────────────── Resposta<Mtr> ──────────────────────────────────┐
│                                                                   │
│  {                                                                │
│    "id": "0193a1b2-...",  // UUID v7                              │
│    "ok": true,                                                    │
│    "dados": [ { "numero": "261032125684", "residuos": [...] } ],  │
│    "erros": [],                                                   │
│    "total": 42              // totalRecords da API                │
│  }                                                                │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
```

---

## 🚀 Quick-start

### Adicione ao `Cargo.toml`:

```toml
[dependencies]
open-mtr-core = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Configuração — SINIR (token de integração):

```rust
use open_mtr_core::*;

let mtr = OpenMtrClient::builder()
    .base(System::Sinir)
    .token(std::env::var("OPENMTR_SINIR_TOKEN")?)
    .build()
    .await?;
```

### Configuração — MTR-PE (CPF + senha):

```rust
use open_mtr_core::*;

// 1. Autentica com CPF+senha+unidade+CNPJ
let token = OpenMtrAuthBuilder::default()
    .base(System::MtrPe)
    .credenciais("07800211428", "NORLU", 3623)
    .cnpj("15805619000140")
    .build()
    .await?;

// 2. Usa o token JWT obtido
let mtr = OpenMtrClient::builder()
    .base(System::MtrPe)
    .token(&token)
    .build()
    .await?;
```

### Consultar MTRs específicos:

```rust
// Retorna dados completos via GET /retornaManifesto/{n}
let resp = mtr.consultar_mtrs(&FiltroMtr {
    numeros: Some(vec!["261032125684".into()]),
    ..Default::default()
}).await?;

for m in &resp.dados {
    println!("#{} status={:?}", m.numero, m.status);
    println!("   gerador: {:?}", m.gerador.as_ref().map(|p| &p.razao_social));
    println!("   resíduos: {}", m.residuos.len());
    for r in &m.residuos {
        println!("     - {} qtd={}", r.codigo_ibama, r.quantidade);
    }
}
```

### Consultar por período + enriquecer:

```rust
// 1 request de lista + N requests paralelos de detalhe
let resp = mtr.consultar_mtrs(&FiltroMtr {
    periodo: Some(Periodo::ultimos_dias(30)),
    parte: Some(FiltroParte {
        tipo: TipoParte::Transportador,
        ..Default::default()
    }),
    detalhado: true,   // ← enriquece com resíduos, CNPJs, etc.
    ..Default::default()
}).await?;

println!("{} MTRs encontrados (total: {:?})", resp.dados.len(), resp.total);
```

### Download em lote:

```rust
let resp = mtr.download_mtrs(vec![
    "261032125684".into(),
    "251032126570".into(),
]).await?;

for item in &resp.dados {
    tokio::fs::write(&item.filename, &item.buffer).await?;
}
```

### Tabelas de apoio:

```rust
let residuos = mtr.listar_residuos().await?;
let acondicionamentos = mtr.listar_acondicionamentos().await?;
let classes = mtr.listar_classes().await?;
let estados = mtr.listar_estados_fisicos().await?;
let tratamentos = mtr.listar_tratamentos().await?;
let unidades = mtr.listar_unidades().await?;
```

### Tratamento de erros (RFC 9457):

```rust
match mtr.consultar_mtrs(&filtro).await {
    Ok(resp) => {
        for m in &resp.dados { println!("✅ {}", m.numero); }
        for e in &resp.erros {
            eprintln!("⚠️  [{}] {}", e.status, e.detail);
        }
    }
    Err(cat) => {
        // Catastrófico: rede, timeout, auth
        eprintln!("❌ {} — {} (type: {})", cat.title, cat.detail, cat.r#type);
    }
}
```

---

## 🗺️ Sistemas Suportados

| Sistema | Status | Adapter | Base URL |
|---|---|---|---|---|
| **SINIR Nacional** | ✅ Completo | `SinirAdapter` | `admin.sinir.gov.br/api` |
| **MTR Pernambuco** | ✅ Completo | `MtrPeAdapter` | `mtr.cprh.pe.gov.br/api` |
| **SIGOR (CETESB/SP)** | ✅ Completo | `SigorAdapter` | `mtrr.cetesb.sp.gov.br/apiws/rest` |
| MTR Ceará | 🚧 Pendente | — | — |
| MTR São Paulo | 🚧 Pendente | — | — |
| MTR Rio de Janeiro | 🚧 Pendente | — | — |
| MTR Minas Gerais | 🚧 Pendente | — | — |
| MTR Bahia | 🚧 Pendente | — | — |

```rust
// Alternar entre sistemas é trocar uma linha:
let mtr = OpenMtrClient::builder()
    .base(System::Sinir)   // ou System::Sigor, System::MtrPe, ...
    .token("...")
    .build()
    .await?;
// Todos os 16 métodos funcionam da mesma forma — mesmo modelo de domínio

// Exemplo: MTR-PE devolve dados tão ricos quanto SINIR
println!("{}", mtr.dados[0].numero);           // 2114014590
println!("{:?}", mtr.dados[0].gerador);        // Some(Parte { documento: 16.701.716/0034-14, unidade: 3581 })
println!("{:?}", mtr.dados[0].residuos[0]);    // Residuo { codigo_ibama: "150103", quantidade: 2.54, ... }
```

---

## 🎯 17 Métodos Universais

### MTRs

| Método | Parâmetro | Retorno | Descrição |
|---|---|---|---|
| `consultar_mtrs` | `&FiltroMtr` | `Resposta<Mtr>` | Consulta por número(s), período, parte ou composição |
| `criar_mtrs` | `Vec<CriarMtr>` | `Resposta<MtrCriado>` | Criar manifestos em lote |
| `receber_mtrs` | `Vec<ReceberMtr>` | `Resposta<MtrRecebido>` | Registrar recebimento em lote |
| `cancelar_mtrs` | `Vec<CancelarMtr>` | `Resposta<MtrCancelado>` | Cancelar em lote |
| `modificar_mtrs` | `Vec<ModificarMtr>` | `Resposta<MtrModificado>` | Solicitar alteração de recebimento |
| `aceitar_alteracoes` | `Vec<AceitarAlteracao>` | `Resposta<AlteracaoAceita>` | Aceitar ou recusar alteração solicitada |
| `download_mtrs` | `Vec<String>` | `Resposta<DownloadItem>` | Download de PDF em lote |

### CDF (Certificado de Destinação Final)

| Método | Parâmetro | Retorno | Descrição |
|---|---|---|---|
| `gerar_cdfs` | `Vec<GerarCdf>` | `Resposta<CdfEmitido>` | Emitir certificados em lote |
| `consultar_cdfs` | `&FiltroCdf` | `Resposta<Cdf>` | Consultar certificados |
| `download_cdfs` | `Vec<DownloadCdf>` | `Resposta<DownloadItem>` | Download PDF/XML em lote |

### Tabelas de Apoio

| Método | Retorno | Descrição |
|---|---|---|
| `listar_residuos` | `Resposta<ResiduoIbama>` | Lista de resíduos IBAMA |
| `listar_acondicionamentos` | `Resposta<Acondicionamento>` | Tipos de acondicionamento |
| `listar_classes` | `Resposta<ClasseResiduo>` | Classes de resíduo |
| `listar_estados_fisicos` | `Resposta<EstadoFisico>` | Estados físicos |
| `listar_tratamentos` | `Resposta<TratamentoResiduo>` | Tipos de tratamento |
| `listar_unidades` | `Resposta<UnidadeMedida>` | Unidades de medida |

### Autenticação Legado (CPF + senha)

| Método | Descrição |
|---|---|
| `OpenMtrAuthBuilder::default().credenciais(cpf, senha, unidade).cnpj(cnpj).build()` | Para sistemas que exigem CPF+senha (ex: MTR-PE) |

> MTR-PE requer CPF, senha, código da unidade **e** CNPJ da empresa.
> O token retornado deve ser passado via `.token()` ao `OpenMtrClient::builder()`.<br>
> Token expira em 1 hora — é responsabilidade da aplicação reautenticar.

---

## 🧩 Adapter Pattern

Cada sistema implementa o trait `MtrAdapter` com **17 métodos**. A facade `OpenMtrClient` delega para o adapter sem saber de paths, DTOs ou particularidades do sistema.

```rust
#[async_trait]
pub(crate) trait MtrAdapter: Send + Sync {
    fn system(&self) -> System;
    fn nome(&self) -> &'static str;

    async fn autenticar_token(&self, ctx: &MtrCtx) -> Result<String, OpenMTRError>;
    async fn autenticar_credenciais(&self, ctx: &MtrCtx, ...) -> Result<String, OpenMTRError>;

    async fn consultar_mtrs(&self, ctx: &MtrCtx, filtro: &FiltroMtr) -> Result<LoteResult<Mtr>, OpenMTRError>;
    async fn criar_mtrs(&self, ctx: &MtrCtx, mtrs: Vec<CriarMtr>) -> Result<LoteResult<MtrCriado>, OpenMTRError>;
    async fn aceitar_alteracoes(&self, ctx: &MtrCtx, reqs: Vec<AceitarAlteracao>) -> Result<LoteResult<AlteracaoAceita>, OpenMTRError>;
    // ... +14 métodos
}
```

### Implementando um novo sistema

```rust
struct MtrPeAdapter;

#[async_trait]
impl MtrAdapter for MtrPeAdapter {
    fn system(&self) -> System { System::MtrPe }
    fn nome(&self) -> &'static str { "MTR PE" }

    async fn consultar_mtrs(&self, ctx: &MtrCtx, filtro: &FiltroMtr) -> Result<LoteResult<Mtr>, OpenMTRError> {
        // 1. Montar requisição no formato do MTR-PE
        // 2. ctx.http.post() / ctx.http.get()
        // 3. Converter resposta JSON para tipos do domínio
        // 4. Retornar LoteResult<Mtr>
    }
    // ... +16 métodos
}
```

### SinirAdapter (implementação de referência — SINIR Nacional)

O adapter SINIR usa **3 rotas de consulta** otimizadas:

| Situação | Endpoint | Resíduos | CNPJ |
|---|---|---|---|
| `numeros: ["N"]` | `GET /retornaManifesto/{N}` | ✅ Completo | ✅ Preenchido |
| `periodo + detalhado: true` | `POST /listaManifesto` → extrai números → N× `GET /retornaManifesto/{N}` | ✅ Completo | ✅ Preenchido |
| `periodo + detalhado: false` (ou omitido) | `POST /listaManifesto` | ❌ Vazio | ❌ `null` |

### MtrPeAdapter (MTR Pernambuco)

O MTR-PE difere do SINIR em vários aspectos — o adapter normaliza tudo:

| Aspecto | SINIR | MTR-PE | OpenMTR (unificado) |
|---|---|---|---|
| **Partes** | `parceiroGerador: {parCnpj, parDescricao}` | `cnpGerador` + `codUnidadeGerador` (plano) | `gerador: Parte { documento, unidade }` |
| **Nº MTR** | `manNumero` (string) | `manifestoCodigo` (integer) | `numero: String` |
| **Datas** | Timestamp ms | String `yyyyMMdd` | `data_criacao: NaiveDate` |
| **Itens** | `listaManifestoResiduo[]` c/ `resCodigoIbama` | `itemManifestoJSONs[]` c/ `residuo` | `residuos: Vec<Residuo>` |
| **Wrapper** | `{erro, objetoResposta, mensagem}` | Array direto ou `{manifestoJSONDtos: [...]}` | `Resposta<T>` c/ envelope RFC 9457 |

**Resultado:** MTR-PE preenche 10/18 campos do modelo unificado (vs 8/18 do SINIR em amostras reais), incluindo `motorista`, `placa` e `data_expedicao` que o SINIR frequentemente não devolve.

---

## 📦 Envelope de Resposta

Toda operação retorna `Result<Resposta<T>, OpenMTRError>` — nunca `Result<Vec<T>>`.

### Sucesso (`Ok(Resposta)`)
```json
{
  "id": "0193a1b2-cdef-7012-b456-789012345678",
  "ok": true,
  "status": 200,
  "mensagem": "OK",
  "dados": [ { "numero": "261032125684", "residuos": [...] } ],
  "erros": [],
  "total": 42
}
```

### Erro de negócio (`Ok(Resposta)` com `erros`)
```json
{
  "id": "0193a1b2-...",
  "ok": true,
  "status": 200,
  "dados": [],
  "erros": [
    { "type": "urn:openmtr:error:not-found",
      "title": "Não encontrado",
      "status": 404,
      "detail": "MTR 999999 não encontrado",
      "instance": "0193a1b2-..." }
  ],
  "total": null
}
```

### Erro catastrófico (`Err(OpenMTRError)`)
```json
{
  "type": "urn:openmtr:error:network",
  "title": "Erro de rede",
  "status": 503,
  "detail": "connection refused after 3 retries",
  "instance": "0193a1b2-..."
}
```

---

## 🔬 Enriquecimento Detalhado

O campo `detalhado: true` no `FiltroMtr` transforma uma consulta simples em uma busca completa com resíduos:

```rust
// Sem detalhado — apenas dados básicos (1 request)
let basico = mtr.consultar_mtrs(&FiltroMtr {
    periodo: Some(Periodo::ultimos_dias(30)),
    ..Default::default()
}).await?;
// basico.dados[0].residuos → Vec::new() (vazio)

// Com detalhado — dados completos (N+1 requests paralelos)
let completo = mtr.consultar_mtrs(&FiltroMtr {
    periodo: Some(Periodo::ultimos_dias(30)),
    detalhado: true,
    ..Default::default()
}).await?;
// completo.dados[0].residuos → [Residuo { codigo_ibama: "170903(*)", ... }]
```

O enriquecimento é feito em **paralelo** — todas as requisições de detalhe disparam simultaneamente via `tokio`.

---

## 🌐 Uso TypeScript / Python

### TypeScript (napi-rs)

```typescript
import { OpenMtrClient } from '@open-mtr/native';

const client = new OpenMtrClient('sinir', process.env.OPENMTR_SINIR_TOKEN!);

// Consulta
const resp = await client.consultarMtrs(JSON.stringify({
  numeros: ['261032125684']
}));
console.log(resp.dados[0].numero, resp.dados[0].residuos.length);

// Download
const pdfs = await client.downloadMtrs(['261032125684']);
```

### Python (UniFFI)

```python
from open_mtr_uniffi import OpenMtrClient

client = OpenMtrClient.init("sinir", "token", None)
result = client.listar_manifestos('{"numeros": ["261032125684"]}')
```

---

## 🧪 Testes

```bash
cargo test -p open-mtr-core                              # testes unitários + wiremock
cargo test -p open-mtr-core --test live_sinir_test       # SINIR real (requer token)
cargo test -p open-mtr-core --test live_mtr_pe_consulta  # MTR-PE real (requer .env)
cargo test -p open-mtr-core --test live_compare_systems  # ambos lado a lado
```

### Cobertura

| Tipo | Qtd | O que testa |
|---|---|---|
| **Unitários** | 68 | Validação de newtypes, construção de `Resposta`, `Periodo`, serialização |
| **Wiremock** | 8 | Consulta por número, consulta por período, erro 404, download PDF |
| **OpenMtrClient** | 5 | Builder (sistema faltando, token faltando), `System::from_str` |
| **Live SINIR** | 14 | Consulta, filtros, tabelas, download, token inválido, detalhado |
| **Live MTR-PE** | 1 | Consulta completa: 50 MTRs reais, validação de estrutura |
| **Live comparativo** | 1 | Ambos sistemas lado a lado, métricas de preenchimento |

### Testes ao vivo

```bash
cp .env.example .env
# SINIR: OPENMTR_SINIR_TOKEN
# MTR-PE: MTR_PE_CNPJ, MTR_PE_CPF_USER, MTR_PE_PASSWORD, MTR_PE_UNITY
cargo test -p open-mtr-core --test live_sinir_test
cargo test -p open-mtr-core --test live_mtr_pe_consulta
cargo test -p open-mtr-core --test live_compare_systems
```

---

## 🛠️ Stack

| Componente | Tecnologia |
|---|---|
| Runtime | Rust 1.85+ (edition 2024) |
| Async | `tokio` (sync, time, fs) |
| HTTP | `reqwest` 0.12 (async, rustls-tls) |
| Serialização | `serde` + `serde_json` |
| Erros | RFC 9457 (Problem Details) + UUID v7 |
| Logs | `tracing` — spans, `#[instrument]`, structured |
| Retry | Backoff exponencial (3 tentativas, 1s/2s/4s) |
| Rate limiting | Sliding window configurável |
| FFI (Node.js) | napi-rs |
| FFI (Python/Kotlin/Swift) | UniFFI |
| Licença | MPL-2.0 |

### Dependências

```toml
[dependencies]
serde = "1"           # Serialização JSON
serde_json = "1"      # Parsing JSON
reqwest = "0.12"      # HTTP client
tokio = "1"           # Async runtime
chrono = "0.4"        # Datas
uuid = { version = "1", features = ["v7"] }    # Correlation IDs
url = "2"             # URLs
tracing = "0.1"       # Logs estruturados
base64 = "0.22"       # Decode JWT
async-trait = "0.1"   # Async trait methods
```

---

## 📋 Comparação: openMTR vs Wrapper Específico

| Aspecto | Wrapper específico (ex: SDK SINIR) | openMTR |
|---|---|---|
| **Sistemas** | 1 (SINIR) | N (SINIR + estaduais) |
| **API pública** | Acoplada aos DTOs do SINIR | Tipos universais de domínio |
| **Adapter** | Inexistente | `MtrAdapter` trait — 1 por sistema |
| **Resposta** | `Result<Vec<T>>` | `Result<Resposta<T>, OpenMTRError>` com envelope RFC 9457 |
| **Cache** | Geralmente embutido | Responsabilidade da aplicação final |
| **Resíduos** | Só na criação | Via `detalhado: true` ou consulta por número |
| **FFI** | N/A | napi-rs (Node.js) + UniFFI (Python/Kotlin/Swift) |

---

<div align="center">

**Licença:** Mozilla Public License v2.0 (MPL-2.0)

</div>
