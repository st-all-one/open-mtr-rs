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

## Autenticação Legado

Para sistemas que ainda exigem CPF + senha:

```rust
let token = OpenMtrAuth::builder()
    .base(System::MtrX)
    .credenciais("cpf", "senha", 123)
    .build()
    .await?;

let mtr = OpenMtrClient::builder()
    .base(System::MtrX)
    .token(token)
    .build()
    .await?;
```

## Exemplo TypeScript

```typescript
const client = new OpenMtrClient('sinir', process.env.TOKEN);
```

## Exemplo Python

```python
client = OpenMtrClient.init("sinir", "token", None)
```
