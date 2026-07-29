# Schemas

Cada domínio da API SINIR mapeado em um módulo separado em `core/src/schemas/`.

## common.rs

Tipos genéricos reusados por todos os domínios.

### ObjetoRespostaRest\<T\>

Wrapper padrão de toda resposta da API SINIR:

```rust
pub struct ObjetoRespostaRest<T> {
    pub erro: bool,
    pub objeto_resposta: Option<T>,    // renomeado de "objetoResposta"
    pub mensagem: Option<String>,
    pub total_records: Option<i64>,     // paginação
}
```

### DataMtr

Newtype sobre `chrono::NaiveDate` com formato `dd-MM-yyyy`:

```rust
pub struct DataMtr(pub chrono::NaiveDate);

DataMtr::new(date)           // de NaiveDate
DataMtr::parse("01-01-2024") // de string
Serialize → "01-01-2024"
Deserialize ← "01-01-2024"
```

### Pagina / TamanhoPagina

Newtypes com validação em `::new()`:

| Tipo | Validação |
|------|-----------|
| `Pagina` | `>= 1` |
| `TamanhoPagina` | `1..=50` |

### Paginacao\<T\>

Struct de resposta com paginação (não usado atualmente — a API retorna lista direta + `total_records`):

```rust
pub struct Paginacao<T> {
    pub content: Vec<T>,
    pub total_pages: i32,
    pub total_elements: i64,
    pub page: i32,
    pub page_size: i32,
}
```

## auth.rs

```rust
pub struct ApiwsLoginDTO    { usuario, senha }           // deprecated
pub struct TokenObjeto      { token, _tipo, expira_em }
pub struct TokenResponse    { erro, objeto, mensagem }
```

## cnpj.rs — Cnpj newtype

```rust
pub struct Cnpj(String);
```

- `Cnpj::new("11.222.333/0001-44")` — valida dígitos verificadores
- `Cnpj::parse_unchecked(s)` — sem validação (para deserialize)
- `Serialize` → string limpa `"11222333000181"`
- `Deserialize` → aceita qualquer string (leniente)
- `Display` → formatado `"11.222.333/0001-81"`

## cpf.rs — Cpf e CpfCnpj

```rust
pub struct Cpf(String);

pub enum CpfCnpj {
    Cpf(Cpf),
    Cnpj(Cnpj),
}
```

- `Cpf::new("529.982.247-25")` — valida dígitos
- `CpfCnpj::new("...")` — 11 dígitos → Cpf, 14 → Cnpj
- Deserialize automático por tamanho

## manifesto.rs

### ApiwsListaManifestoDTO

```rust
pub struct ApiwsListaManifestoDTO {
    pub data_inicial: DataMtr,
    pub data_final: DataMtr,
    pub tipo_parceiro: TipoParceiro,  // enum: Transportador=5, Gerador=8, ...
    pub pagina: Option<i32>,
    pub tamanho_pagina: Option<i32>,
}
```

### ManifestoResumo (~40 campos)

Inclui: `man_numero`, `man_status`, parceiros, resíduos, datas, flags, campos internacionais (Option).

### Sub-structs

| Struct | Campos |
|--------|--------|
| `Estado` | `abreviacao`, `codigo` |
| `Pais` | `codigo`, `descricao` |
| `ParceiroAcesso` | `est_codigo`, `paa_codigo`, `paa_cpf: Cpf`, `paa_nome` |
| `SituacaoManifesto` | `codigo`, `data_recebimento`, `descricao`, `ordem` |
| `ParceiroMTR` | `cidade`, `cnpj: Option<Cnpj>`, `codigo`, `descricao`, ... |
| `ManifestoResiduo` | `res_codigo_ibama`, `quantidade`, `uni_codigo`, classificações |
| `ManifestoResiduoVariacao` | `sequencial`, `quantidade`, `motivo` |

### Cancelamento

```rust
pub struct CancelaManifestoDTO { man_numero, justificativa }
impl CancelaManifestoDTO {
    pub fn validate(&self) -> Result<(), MtrError>  // justificativa >= 20 chars
}
pub struct CancelamentoResponse { man_numero, status, data_cancelamento }
```

## recebimento.rs

| Struct | Função |
|--------|--------|
| `TimestampMs(i64)` | Newtype timestamp em ms, Serialize/Deserialize |
| `ResiduoRecebido` | `res_codigo_ibama`, `quantidade_recebida`, `uni_codigo`, classificação |
| `ApiwsManifestoDeRecebimentoDTO` | Payload de recebimento |
| `RecebimentoLoteItem` | Item na resposta |
| `ResiduoAlterado` | Resíduo com `nova_quantidade` |
| `ApiwsAlteraRecebimentoDTO` | Payload de alteração |
| `AceiteAlteracaoDTO` | Aceite (booleano) + observações |
| `AceiteAlteracaoResponse` | Resposta do aceite |

## cdf.rs

| Struct | Função |
|--------|--------|
| `ApiwsCertificadoDestinacaoDTO` | Payload de emissão |
| `CDFEmissaoObjeto` | `cdf_codigo`, `cdf_numero`, `status` |
| `CDFEmissaoResponse` | Wrapper de resposta |
| `FormatoDownload` | Enum `Pdf` / `Xml` |

Função `validar_emissao()`: valida lista não vazia, data não futura, data ≤ 90 dias atrás.

## logistica_reversa.rs

| Struct | Função |
|--------|--------|
| `MtrLrDataDTO` | Filtro por período |
| `MtrOrigemCompletoDTO` | Consulta por `man_numero` |
| `RastreioEvento` | Evento individual no rastreio |
| `MtrInfo` | Resumo do MTR |
| `MtrOrigemCompletoObjeto` | MTR + rastreio |
| `MtrProcessoCompletoDTO` | Consulta de processo |
| `ApiwsAlteraManifestoOrigemDTO` | Alterar titularidade |

## integracao_estadual.rs

| Struct | Função |
|--------|--------|
| `ParteMTR` | `cnpj: Cnpj`, `razao_social`, endereço |
| `ResiduoIntegracao` | Resíduo no contexto IE |
| `ApiwsManifestoDTO` | Manifesto completo (gerador + transportador + destinador) |
| `ErroIntegracao` | Erro individual na resposta |
| `RespostaIntegracaoObjeto` | `manifestos_processados`, `manifestos_com_erro`, `erros` |

## tabelas.rs

| Struct | Função |
|--------|--------|
| `TipoAcondicionamento` | `codigo`, `descricao` |
| `UnidadeMedida` | `codigo`, `descricao`, `sigla` |
| `EstadoFisico` | `codigo`, `descricao` |
| `ClasseResiduo` | `codigo`, `descricao`, `resolucao` |
| `TratamentoResiduo` | `codigo`, `descricao` |

### LazyLock maps

```rust
pub static COMPATIBILIDADE_ACOND: LazyLock<HashMap<i32, Vec<i32>>>
pub static ESTADOS_FISICOS: LazyLock<HashMap<i32, &str>>
```
