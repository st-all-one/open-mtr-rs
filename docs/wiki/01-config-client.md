# Configuração — OpenMtrClient + Builder

## OpenMtrClient

Ponto de entrada único do openMTR. Todas as operações partem daqui.

```rust
use open_mtr_core::*;

let mtr = OpenMtrClient::builder()
    .base(System::Sinir)
    .token("meu-integration-token")
    .build()
    .await?;
```

## OpenMtrClientBuilder

| Método | Obrigatório | Descrição |
|--------|-------------|-----------|
| `.base(System::...)` | ✅ | Sistema alvo (Sinir, Sigor, MtrPe, etc.) |
| `.token("...")` | ✅ | Integration token JWT |
| `.base_url("...")` | ❌ | Override da URL base (default do System) |
| `.timeout_secs(n)` | ❌ | Timeout em segundos (default: 60) |
| `.max_rps(n)` | ❌ | Requisições por segundo (default: 10) |

## Sistema

```rust
System::Sinir   // Nacional — admin.sinir.gov.br/api
System::Sigor   // SIGOR (Ceará)
System::MtrPe   // MTR Pernambuco
System::MtrCe   // MTR Ceará
// + MtrSp, MtrRj, MtrMg, MtrBa
```

## Autenticação Legado (CPF + senha)

Para sistemas que ainda exigem CPF + senha (ex.: MTR-PE, SIGOR), use o
`OpenMtrClassicAuth` — um facade isolado que consome as credenciais e retorna
**sempre o token JWT**:

```rust
use open_mtr_core::*;

let params = AuthParams::CpfSenhaUnidade {
    cpf: "07800211428".into(),
    senha: "minha-senha".into(),
    unidade: 3615,
    cnpj: Some("11201870000226".into()), // obrigatório no MTR-PE
};

// Atalho funcional:
let token = OpenMtrClassicAuth::obter_token(System::MtrPe, &params).await?;

// Ou via builder:
let token = OpenMtrClassicAuth::new(System::MtrPe)
    .params(params)
    .timeout_secs(60)
    .build()
    .await?;

let mtr = OpenMtrClient::builder()
    .base(System::MtrPe)
    .token(token)
    .build()
    .await?;
```

O `AuthParams` serializa como JSON (`{"tipo":"cpf_senha_unidade",...}`), então o
mesmo facade funciona nos bindings (WASM/NAPI/UniFFI) — ex.: no Deno SSR:

```typescript
const auth = JSON.parse(await client.obter_token("mtr_pe", JSON.stringify(params)));
await client.init("mtr_pe", auth.token, "");
```

## Exemplo TypeScript

```typescript
const client = new OpenMtrClient('sinir', process.env.TOKEN);
```

## Exemplo Python

```python
client = OpenMtrClient.init("sinir", "token", None)
```
