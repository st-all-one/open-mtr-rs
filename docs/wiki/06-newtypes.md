# Domain Newtypes

Newtypes com validação para tipos primitivos do domínio SINIR.

## Cnpj

```rust
pub struct Cnpj(String);
```

### Validação (`Cnpj::new`)
1. Filtra só dígitos ASCII
2. Exatamente 14 dígitos
3. Rejeita dígitos repetidos (ex: `11.111.111/1111-11`)
4. Calcula e confere 1º e 2º dígitos verificadores

### Serialização
- `Serialize` → string limpa `"11222333000181"`
- `Deserialize` → aceita qualquer string sem validação (leniente)
- `Display` → formatado `"11.222.333/0001-81"`

### Uso

```rust
// Validação estrita (input do usuário)
let cnpj = Cnpj::new("11.222.333/0001-44")?;

// Deserialize da API (aceita o que vier)
let cnpj: Cnpj = serde_json::from_str(r#""11222333000181""#)?;
```

### Campos que usam Cnpj

| Schema | Campo |
|--------|-------|
| `ParceiroMTR` | `cnpj: Option<Cnpj>` |
| `ParteMTR` | `cnpj: Cnpj` |

## Cpf

```rust
pub struct Cpf(String);
```

Validação similar ao Cnpj: 11 dígitos, check digits, sem repetição.

### Campos que usam Cpf

| Schema | Campo |
|--------|-------|
| `ParceiroAcesso` | `paa_cpf: Cpf` |

## CpfCnpj

```rust
pub enum CpfCnpj {
    Cpf(Cpf),
    Cnpj(Cnpj),
}
```

- `CpfCnpj::new("...")` → detecta por tamanho (11→Cpf, 14→Cnpj)
- `Deserialize` → automático por len

### Campos que usam CpfCnpj

| Schema | Campo |
|--------|-------|
| `ParteManifesto` | `cpf_cnpj: CpfCnpj` |

## DataMtr

```rust
pub struct DataMtr(pub chrono::NaiveDate);
```

Formato SINIR: `dd-MM-yyyy`

```rust
DataMtr::new(date)                    // de NaiveDate
DataMtr::parse("01-01-2024")?         // de string
Serialize  → "01-01-2024"
Deserialize ← "01-01-2024"
```

## TimestampMs

```rust
pub struct TimestampMs(pub i64);      // ms desde epoch
```

```rust
TimestampMs::now()
TimestampMs::from_datetime(&dt)
Serialize  → i64
Deserialize ← i64
```

## Pagina

```rust
pub struct Pagina(u32);
Pagina::new(1)   // Ok
Pagina::new(0)   // Err — "pagina deve ser >= 1"
```

## TamanhoPagina

```rust
pub struct TamanhoPagina(u32);
TamanhoPagina::new(1)    // Ok
TamanhoPagina::new(50)   // Ok
TamanhoPagina::new(51)   // Err — "tamanho da pagina deve estar entre 1 e 50"
```

## Regra de Ouro

- `::new()` — validação estrita (input do usuário)
- `Deserialize` — leniente (aceita o que a API retornar)
- `::parse_unchecked()` / `::from()` — sem validação
