# Newtypes com Validação

## Cnpj

```rust
Cnpj::new("11.222.333/0001-81")?;     // formatado
Cnpj::new("11222333000181")?;         // limpo
```

- 14 dígitos com validação de dígitos verificadores
- Rejeita sequências repetidas (`00.000.000/0000-00`)
- Serializa como `"11.222.333/0001-81"` (formatado)
- Deserialize aceita ambos os formatos

## Cpf

```rust
Cpf::new("529.982.247-25")?;          // formatado
Cpf::new("52998224725")?;             // limpo
```

- 11 dígitos com validação de dígitos verificadores
- Rejeita sequências repetidas

## CpfCnpj

Auto-detecta pelo tamanho: 11 → CPF, 14 → CNPJ.

```rust
let doc = CpfCnpj::new("52998224725")?;   // CpfCnpj::Cpf(...)
let doc = CpfCnpj::new("11222333000181")?; // CpfCnpj::Cnpj(...)
```

## DataMtr

Wrapper sobre `NaiveDate` com formato `dd-MM-yyyy`.

```rust
let data = DataMtr::parse("01-01-2025")?;
let data = DataMtr::new(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
```

Serializa como `"01-01-2025"`.

## TimestampMs

Milissegundos desde epoch (Unix timestamp em ms).

```rust
let ts = TimestampMs::now();
let ts = TimestampMs::from_datetime(naive_datetime);
```
