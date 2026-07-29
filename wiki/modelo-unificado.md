# Modelo Unificado OpenMTR

## Filosofia

O OpenMTR não cria um DTO diferente para cada sistema. Em vez disso, define **um único conjunto de tipos de domínio** (`Mtr`, `Parte`, `Residuo`, etc.) que cada adapter preenche conforme a capacidade da API de origem.

Campos que um sistema não oferece simplesmente ficam como `None` — sem perda de informação, sem acoplamento.

## Estrutura do `Mtr`

```rust
pub struct Mtr {
    pub numero: String,
    pub status: StatusMtr,
    pub gerador: Option<Parte>,
    pub transportador: Option<Parte>,
    pub destinador: Option<Parte>,
    pub residuos: Vec<Residuo>,
    pub data_criacao: Option<NaiveDate>,
    pub data_recebimento: Option<NaiveDate>,
    pub observacao: Option<String>,
    pub motorista: Option<String>,
    pub placa: Option<String>,
    pub tipo_manifesto: Option<i32>,
    pub estado_uf: Option<String>,
    pub data_expedicao: Option<NaiveDate>,
    pub armazenamento_temporario: Option<bool>,
    pub hash: Option<String>,
    pub tem_alteracao_pendente: Option<bool>,
    pub numero_estadual: Option<String>,
    pub codigo_estadual: Option<i64>,
}
```

## Comparação Real — SINIR vs MTR-PE

Testado em 29/07/2026 contra APIs reais de ambos os sistemas:

| Campo | SINIR #1 | SINIR #2 | MTR-PE #1 | MTR-PE #2 |
|-------|----------|----------|-----------|-----------|
| `numero` | `251032129363` | `251032129210` | `2114014590` | `2114015467` |
| `status` | `Ativo` | `Cancelado` | `Ativo` | `Ativo` |
| `gerador.documento` | `15.805.619/0001-40` | `20.300.157/0034-08` | `16.701.716/0034-14` | `10.892.164/0001-24` |
| `gerador.unidade` | `130204` | `453479` | `3581` | `1159` |
| `residuos[].codigo_ibama` | `200199` | `Classe B` | `150103` | `030308`, `200139` |
| `residuos[].quantidade` | `1.0` | `1.0` | `2.54` | `1.207`, `0.588` |
| `data_criacao` | `2026-07-29` | `2026-07-29` | `2026-06-30` | `2026-06-30` |
| `data_expedicao` | ❌ | ❌ | `2026-06-30` | `2026-06-30` |
| `motorista` | ❌ | ❌ | `MARCELO DA SILVA` | `NILSON` |
| `placa` | ❌ | ❌ | `TRP2I76` | `ELX1A42` |
| `estado_uf` | `PB` | `PB` | ❌ | ❌ |

**Métrica:** MTR-PE preencheu **10/18** campos, SINIR preencheu **8/18** — o MTR-PE contribui com `motorista`, `placa` e `data_expedicao` que o SINIR não devolveu nestes registros.

## `Parte` — Estrutura Flexível

```rust
pub struct Parte {
    pub documento: CpfCnpj,    // CPF (11 dig) ou CNPJ (14 dig)
    pub razao_social: String,  // Nome da empresa/pessoa
    pub endereco: Option<String>,
    pub municipio: Option<String>,
    pub uf: Option<String>,
    pub unidade: Option<u32>,  // Código da unidade
}
```

| Fonte | documento | razao_social | unidade | endereço/UF |
|-------|-----------|-------------|---------|-------------|
| SINIR (detalhado) | ✅ | ✅ | ✅ | ✅ |
| SINIR (lista) | ✅ | ✅ | ✅ | ❌ |
| MTR-PE | ✅ | ❌ | ✅ | ❌ |

## `Residuo` — Itens do Manifesto

```rust
pub struct Residuo {
    pub codigo_ibama: String,
    pub descricao: Option<String>,
    pub quantidade: f64,
    pub unidade: Option<u32>,
    pub estado_fisico: Option<u32>,
    pub classe: Option<u32>,
    pub acondicionamento: Option<u32>,
    pub tratamento: Option<u32>,
}
```

Ambos os sistemas preenchem os mesmos campos de resíduo, com diferenças nos nomes dos campos na API bruta:

| Campo conceitual | SINIR | MTR-PE |
|-----------------|-------|--------|
| Código IBAMA | `resCodigoIbama` | `residuo` |
| Quantidade | `marQuantidade` | `quantidade` |
| Unidade | `uniCodigo` | `codigoUnidade` |
| Estado físico | `tieCodigo` | `codigoTipoEstado` |
| Classe | `claCodigo` | `codigoClasse` |
| Acondicionamento | `tiaCodigo` | `codigoAcondicionamento` |
| Tratamento | `traCodigo` | `codigoTecnologia` |
