# Adapter MTR-PE — Implementação

## Visão Geral

O `MtrPeAdapter` implementa o trait `MtrAdapter` para o sistema MTR de Pernambuco, hospedado em `https://mtr.cprh.pe.gov.br/api`.

## Autenticação

Diferente do SINIR (que usa `integration_token`), o MTR-PE exige autenticação via CPF + senha:

```
POST /gettoken
{
  "pessoaCodigo": 3623,
  "pessoaCnpj": "15805619000140",
  "usuarioCpf": "07800211428",
  "senha": "NORLU"
}
```

O token JWT retornado expira em 1 hora. Não há endpoint de refresh — a aplicação deve reautenticar.

No `OpenMtrClient`, o token é armazenado diretamente via `set_token_direct()` (sem refresh automático).

## Diferenças Críticas vs SINIR

### 1. Prefixo dos Campos

**Documentação errada:** `manifestObservacao`, `manifestGeradorNomeResponsavel` (prefixo `manifest`)
**Real:** `manifObservacao`, `manifGeradorNomeResponsavel` (prefixo `manif`, 5 letras)

Usar o prefixo errado causa erro `"Not able to deserialize data provided"` (erro de desserialização Jackson).

### 2. Estrutura das Partes

| Sistema | Formato |
|---------|---------|
| SINIR | `parceiroGerador: { parCnpj, parDescricao, parEndereco, ... }` |
| MTR-PE | `cnpGerador: "15805619000140"` + `codUnidadeGerador: 3623` (campos planos) |

O adapter normaliza para `Parte { documento, unidade }`. MTR-PE não devolve `razao_social`, `endereco` ou `uf` na consulta de manifesto — esses campos ficam como `None`.

### 3. Datas

| Sistema | Formato | Exemplo |
|---------|---------|---------|
| SINIR | Timestamp ms (i64) | `1711400000000` |
| MTR-PE | String `yyyyMMdd` | `"20260630"` |

### 4. Wrapper de Resposta

| Sistema | Listagem | Criação |
|---------|----------|---------|
| SINIR | `{ erro, objetoResposta, mensagem }` | `{ erro, objetoResposta }` |
| MTR-PE | Array direto (tabelas) | `{ manifestoJSONDtos: [...] }` |

### 5. Nomes de Endpoints

| Descrição | Doc errada | Correto |
|-----------|-----------|---------|
| Receber manifesto | `receiverManifestoLote` | `receberManifestoLote` |
| Emitir CDF — campo | `datalnicial` | `dataInicial` |
| Consultar CDF — campo | `cdfDatalnicial` | `cdfDataInicial` |
| Lista barcodes | `retornaListaCodigoBarrasManifesto` | `retornaListaCodigoBarasManifesto` (1 R) |

### 6. Nomes de Campos nas Tabelas

| Tabela | Doc | Real |
|--------|-----|------|
| Classes | `tpcLaCodigo` | `tpclaCodigo` |
| Estado Físico | `tpostCodigo` | `tpestCodigo` |
| Resíduo | (com acento `Resíduo`) | `retornaListaResiduo` (sem acento) |

## Enriquecimento do Modelo

O adapter preenche 14 dos 19 campos do `Mtr` unificado (o novo campo `barcode` é
preenchido apenas quando a API fornece o código — sistemas como SINIR/SIGOR o
deixam vazio):

| Campo | Preenchido | Fonte |
|-------|-----------|-------|
| `numero` | ✅ | `manifestoCodigo` (i64 → string) |
| `barcode` | ✅ | Código de barras completo (34 dígitos) usado na consulta/download |
| `status` | ✅ | `situacaoManifestoCodigo` (1/3/4/9) |
| `gerador` | ✅ | `cnpGerador` + `codUnidadeGerador` |
| `transportador` | ✅ | `cnpTransportador` + `codUnidadeTransportador` |
| `destinador` | ✅ | `cnpDestinador` + `codUnidadeDestinador` |
| `residuos[]` | ✅ | `itemManifestoJSONs[]` — 8 campos mapeados |
| `data_criacao` | ✅ | `manifData` (parse `yyyyMMdd`) |
| `data_expedicao` | ✅ | `manifDataExpedicao` |
| `data_recebimento` | ✅ | `manifDataRecebimento` |
| `observacao` | ✅ | `manifObservacao` |
| `motorista` | ✅ | `manifTransportadorNomeMotorista` |
| `placa` | ✅ | `manifTransportadorPlacaVeiculo` |
| `armazenamento_temporario` | ✅ | `cnpArmazenador != null` |
| `tipo_manifesto` | ❌ | Não disponível |
| `estado_uf` | ❌ | Não disponível |
| `hash` | ❌ | Não disponível |
| `numero_estadual` | ❌ | Não disponível |
| `codigo_estadual` | ❌ | Não disponível |

## Endpoints Mapeados

| Operação | Endpoint | Wrapper |
|----------|----------|---------|
| Autenticar | `POST /gettoken` | Direto |
| Listar classes | `POST /retornaListaClasse` | Array |
| Listar unidades | `POST /retornaListaUnidade` | Array |
| Listar tecnologias | `POST /retornaListaTecnologia` | Array |
| Listar estados físicos | `POST /retornaListaEstadoFisico` | Array |
| Listar resíduos | `POST /retornaListaResiduo` | Array |
| Listar acondicionamentos | `POST /retornaListaAcondicionamento` | Array |
| Salvar manifesto | `POST /salvarManifestoLote` | `{ manifestoJSONDtos: [...] }` |
| Cancelar manifesto | `POST /cancelarManifesto` | Direto |
| Receber manifesto | `POST /receberManifestoLote` | `{ manifestoRecebimentoJSONs: [...] }` |
| Alterar recebimento | `POST /alteraRecebimento` | `{ manifestoAlterarJSONs: [...] }` |
| Aceitar alteração | `POST /aceiteAlteracaoRecebimento` | Direto |
| Emitir CDF | `POST /emiteCDF` | Direto |
| Consultar CDF | `POST /consultaListaCdf` | Direto |
| Consultar manifesto | `POST /retornaManifesto/{barcode}` | Direto |
| Listar barcodes | `POST /retornaListaCodigoBarasManifesto/{ini}/{fim}` | Direto |
| Download MTR PDF | `POST /buscaPdfManifestoPorCodigoBarras/{barcode}` | Binário |
| Download CDF PDF | `POST /buscaPdfCdf/{codigo}` | Binário |

## Consulta por período (`consultar_mtrs` com `periodo`)

O MTR-PE **não** possui listagem estruturada de manifestos. O endpoint
`retornaListaCodigoBarasManifesto` devolve apenas códigos de barras, com janela
máxima de 30 dias. O método genérico `consultar_mtrs` implementa essa lógica:

1. Percorre o período em janelas de 30 dias chamando `retornaListaCodigoBarasManifesto/{ini}/{fim}`;
2. Deduplica os códigos de barras coletados;
3. Busca o detalhe de cada um via `retornaManifesto/{barcode}`, em paralelo
   (concorrência limitada a 10 requisições);
4. Devolve o `LoteResult<Mtr>` com `total` = quantidade de barcodes únicos.

Cada `Mtr` retornado traz o campo `barcode` preenchido com o código de barras
completo (34 dígitos), pronto para consultas futuras e download do PDF.

Filtro por `numeros` continua suportado (consulta direta por barcode/número).

## Download de PDFs (`download_mtrs`)

O download do PDF usa `POST /buscaPdfManifestoPorCodigoBarras/{barcode}` — exige
o **barcode completo** (34 dígitos), não o `manifestoCodigo` curto. Como o campo
`barcode` fica fixado no `Mtr`, basta:

```rust
let nums: Vec<String> = mtrs.iter().map(|m| m.barcode.clone()).collect();
let resp = client.download_mtrs(nums).await?;
```

Exemplos completos: `core/examples/indexar_mtr_pe.rs` (gera o índice JSON com
barcodes) e `core/examples/baixar_mtr_pe_pdfs.rs` (baixa os PDFs em `mtrs/pe/`).

## Exemplo de Uso

```rust
use open_mtr_core::*;

// Autenticação
let token = OpenMtrAuthBuilder::default()
    .base(System::MtrPe)
    .credenciais("07800211428", "NORLU", 3623)
    .cnpj("15805619000140")
    .build()
    .await?;

// Cliente
let mtr = OpenMtrClient::builder()
    .base(System::MtrPe)
    .token(&token)
    .build()
    .await?;

// Consultar MTRs por número
let resp = mtr.consultar_mtrs(&FiltroMtr {
    numeros: Some(vec!["2114014590".into()]),
    ..Default::default()
}).await?;

for m in &resp.dados {
    println!("#{} status={:?}", m.numero, m.status);
    if let Some(g) = &m.gerador {
        println!("  gerador: {} unidade={:?}", g.documento, g.unidade);
    }
    for r in &m.residuos {
        println!("  resíduo: {} qtd={}", r.codigo_ibama, r.quantidade);
    }
}
```

## Códigos de Situação do Manifesto

| Código | Situação |
|--------|----------|
| 1 | Salvo (Ativo) |
| 3 | Recebido |
| 4 | Cancelado |
| 9 | Em Armazenamento Temporário |

## Arquivos

- `core/src/openmtr/adapters/mtr_pe.rs` — Implementação do adapter (~680 linhas)
- `core/tests/live_mtr_pe_consulta.rs` — Testes de consulta contra API real
- `core/tests/live_compare_systems.rs` — Comparação SINIR vs MTR-PE
- `wiki/mtr-pe/webService.md` — Documentação completa da API
