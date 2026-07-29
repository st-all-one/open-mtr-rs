# Tipos de Domínio

Tipos universais que representam conceitos do MTR independente do sistema.

## Módulos

| Módulo | Tipos |
|--------|-------|
| `mtr.rs` | `Mtr`, `StatusMtr`, `FiltroMtr`, `Periodo`, `FiltroParte` |
| `parte.rs` | `Parte` (gerador/transportador/destinador) |
| `residuo.rs` | `Residuo`, `ResiduoCriacao`, `ResiduoRecebido`, `ResiduoIbama` |
| `newtypes.rs` | `Cnpj`, `Cpf`, `CpfCnpj`, `DataMtr`, `TimestampMs` |
| `criacao.rs` | `CriarMtr`, `MtrCriado` |
| `recebimento.rs` | `ReceberMtr`, `MtrRecebido` |
| `cancelamento.rs` | `CancelarMtr`, `MtrCancelado` |
| `modificacao.rs` | `ModificarMtr`, `MtrModificado`, `ModificacaoTipo` |
| `download.rs` | `DownloadItem`, `DownloadCdf` |
| `cdf.rs` | `GerarCdf`, `Cdf`, `CdfEmitido`, `FiltroCdf` |
| `tabela.rs` | `Acondicionamento`, `UnidadeMedida`, `ClasseResiduo`, `EstadoFisico`, `TratamentoResiduo` |
| `tipo.rs` | `TipoParte`, `TipoManifesto`, `FormatoDownload` |

## Convenções

- Todos os tipos: `Debug, Clone, Serialize, Deserialize`
- Campos opcionais: `Option<T>` com `#[serde(skip_serializing_if = "Option::is_none")]`
- Datas: `DataMtr` (formato `dd-MM-yyyy`) ou `NaiveDate`
- Timestamps: `TimestampMs` (ms from epoch)

## FiltroMtr

```rust
FiltroMtr {
    numeros: Option<Vec<String>>,     // MTRs específicos
    periodo: Option<Periodo>,         // dataInicial/dataFinal
    parte: Option<FiltroParte>,       // filtro por parte
    pagina: Option<u32>,
    tamanho: Option<u32>,
}

Periodo {
    inicio: DataMtr,
    fim: DataMtr,
}
```

Serializa como:
```json
{
  "numeros": ["123456"],
  "periodo": { "dataInicial": "01-01-2025", "dataFinal": "31-01-2025" },
  "parte": { "tipo": "Transportador" }
}
```

## Mtr

```rust
Mtr {
    numero: String,
    status: StatusMtr,
    gerador: Option<Parte>,
    transportador: Option<Parte>,
    destinador: Option<Parte>,
    residuos: Vec<Residuo>,
    data_criacao: Option<NaiveDate>,
    data_recebimento: Option<NaiveDate>,
}
```

## Parte

```rust
Parte {
    documento: CpfCnpj,      // Cpf | Cnpj (auto-detect)
    razao_social: String,
    endereco: Option<String>,
    municipio: Option<String>,
    uf: Option<String>,
    unidade: Option<u32>,
}
```
