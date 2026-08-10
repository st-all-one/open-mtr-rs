# Adapter SIGOR-SP (CETESB)

## Visão Geral

O `SigorAdapter` implementa o trait `MtrAdapter` para o sistema SIGOR-MTR da CETESB (São Paulo), hospedado em `https://mtrr.cetesb.sp.gov.br/apiws/rest`.

O SIGOR-SP é o sistema oficial de gerenciamento de resíduos do Estado de São Paulo. Possui uma API REST com wrapper de resposta `{ mensagem, objetoResposta, erro }` (similar ao SINIR).

## Autenticação

Diferente do SINIR (token de integração) e MTR-PE (pessoaCodigo + pessoaCnpj + usuarioCpf + senha), o SIGOR usa:

```
POST /gettoken
{
  "cpfCnpj": "07800211428",   // CPF do usuário (11 dígitos)
  "senha": "********",
  "unidade": "189788"          // Código da unidade (string)
}
```

O campo `cpfCnpj` **deve** ser o CPF do usuário (11 dígitos). Usar o CNPJ da empresa retorna erro 001.

**Resposta:**
```json
{
  "mensagem": "Sucesso!",
  "objetoResposta": "Bearer eyJhbGciOiJIUzUxMiJ9...",
  "erro": false
}
```

O token JWT vem com prefixo `Bearer ` no campo `objetoResposta`. Extrair o valor sem o prefixo.

## Diferenças Críticas vs SINIR/MTR-PE

### 1. Método HTTP das Listagens

| Sistema | Método |
|---------|--------|
| SINIR | GET |
| MTR-PE | POST |
| **SIGOR** | **GET** (doc errada diz POST) |

### 2. Wrapper de Resposta

| Sistema | Listagens | Criação |
|---------|-----------|---------|
| SINIR | `{ erro, objetoResposta, mensagem }` | `{ erro, objetoResposta }` |
| MTR-PE | Array direto | `{ manifestoJSONDtos: [...] }` |
| **SIGOR** | `{ erro, objetoResposta, mensagem }` | `{ restResponseValido, restResponseMensagem, ... }` |

### 3. Estrutura das Partes no MTR

| Sistema | Formato |
|---------|---------|
| SINIR | `parceiroGerador: { parCnpj, parDescricao, parEndereco, ... }` |
| MTR-PE | `cnpGerador: "..." + codUnidadeGerador: N` (campos planos) |
| **SIGOR** | `parceiroGerador: { parCnpj, parDescricao, parCodigo }` (igual SINIR) |

### 4. Datas

| Sistema | Formato | Exemplo |
|---------|---------|---------|
| SINIR | Timestamp ms (i64) | `1711400000000` |
| MTR-PE | String `yyyyMMdd` | `"20260630"` |
| **SIGOR** | Timestamp ms (i64) | `1711400000000` |

### 5. Nomes de Campos Corrigidos

| Documentação | Real |
|-------------|------|
| `resCodigolbama` (L minúsculo) | `resCodigoIbama` (I maiúsculo) |
| `cerDatalnicial` (L minúsculo) | `cerDataInicial` (I maiúsculo) |
| `receiverManifestoLote` | `receberManifestoLote` |
| `aceiteAlteraRecebimento` | `aceiteAlteracaoRecebimento` |
| `EstadoFísico` (com acento) | `EstadoFisico` (sem acento) |

## Enriquecimento do Modelo

O adapter preenche **15+ dos 18 campos** do `Mtr` unificado:

| Campo | Preenchido | Fonte |
|-------|-----------|-------|
| `numero` | ✅ | `manNumero` |
| `status` | ✅ | `situacaoManifesto.simDescricao` |
| `gerador` | ✅ | `parceiroGerador` (parCnpj, parDescricao, parCodigo) |
| `transportador` | ✅ | `parceiroTransportador` |
| `destinador` | ✅ | `parceiroDestinador` |
| `residuos[]` | ✅ | `listaManifestoResiduo[]` — 16 campos mapeados |
| `data_criacao` | ✅ | `manData` (timestamp ms) |
| `data_expedicao` | ✅ | `manDataExpedicao` |
| `data_recebimento` | ✅ | `dataRecebimentoAT` (formato BR ou timestamp) |
| `observacao` | ✅ | `manObservacao` |
| `motorista` | ✅ | `manNomeMotorista` |
| `placa` | ✅ | `manPlacaVeiculo` |
| `estado_uf` | ✅ | `estado.estAbreviacao` |
| `tipo_manifesto` | ✅ | `tipoManifesto` |
| `armazenamento_temporario` | ✅ | `possuiArmazenamentoTemporario` |
| `numero_estadual` | ✅ | `manNumeroEstadual` |
| `codigo_estadual` | ✅ | `manCodigoEstadual` |
| `hash` | ✅ | `manHashCode` |

**Campos extras de resíduo (enriquecimento):**

| Campo | Preenchido |
|-------|-----------|
| `quantidade_recebida` | ✅ |
| `densidade` | ✅ |
| `numero_onu` | ✅ |
| `classe_risco` | ✅ |
| `nome_embarque` | ✅ |
| `grupo_embalagem` | ✅ |
| `codigo_interno` | ✅ (fallback `codigolinterno` — typos reais da API) |
| `descricao_interna` | ✅ (fallback `descricaolnterna` — typos reais da API) |

## Endpoints Mapeados

| Operação | Método | Endpoint | Wrapper |
|----------|--------|----------|---------|
| Autenticar | POST | `/gettoken` | `{ mensagem, objetoResposta, erro }` |
| Listar classes | **GET** | `/retornaListaClasse` | `{ mensagem, objetoResposta, erro }` |
| Listar unidades | **GET** | `/retornaListaUnidade` | `{ mensagem, objetoResposta, erro }` |
| Listar tratamentos | **GET** | `/retornaListaTratamento` | `{ mensagem, objetoResposta, erro }` |
| Listar estados físicos | **GET** | `/retornaListaEstadoFisico` | `{ mensagem, objetoResposta, erro }` |
| Listar acondicionamentos | **GET** | `/retornaListaAcondicionamento` | `{ mensagem, objetoResposta, erro }` |
| Listar resíduos | **GET** | `/retornaListaResiduo` | `{ mensagem, objetoResposta, erro }` |
| Listar classes por resíduo | **GET** | `/retornaListaClassePorResiduo/{cod}` | `{ mensagem, objetoResposta, erro }` |
| Listar acond. por estado físico | **GET** | `/retornaListaAcondicionamentoPorEstadoFisico/{cod}` | `{ mensagem, objetoResposta, erro }` |
| Consultar MTR | GET | `/retornaManifesto/{num}` | `{ mensagem, objetoResposta, erro }` |
| Consultar MTR por código ref. | POST | `/retornaManifestoSeuCodigo` | Direto |
| Criar MTR | POST | `/salvarManifestoLote` | Direto |
| Criar manifesto complementar | POST | `/salvarManifestoComplementar` | Direto |
| Receber MTR | POST | `/receberManifestoLote` | Direto |
| Cancelar MTR | POST | `/cancelarManifesto` | `{ mensagem, objetoResposta, erro }` |
| Cancelar MTR em lote | POST | `/cancelarManifestoLote` | Direto |
| Alterar recebimento | POST | `/alteraRecebimento` | Direto |
| Aceitar alteração | POST | `/aceiteAlteracaoRecebimento` | Direto |
| Emitir CDF | POST | `/emiteCDF` | Direto |
| Download MTR PDF | POST | `/downloadManifesto/{num}` | Binário |
| Download CDF PDF | POST | `/downloadCertificado/{num}` | Binário |

## Exemplo de Uso

### Autenticação por token de integração (JWT direto)

```rust
use open_mtr_core::*;

let client = OpenMtrClient::builder()
    .base(System::Sigor)
    .token("eyJhbGciOiJIUzUxMiJ9...")
    .build()
    .await?;

// Listar classes
let classes = client.listar_classes().await?;
for c in &classes.dados {
    println!("{}: {}", c.codigo, c.descricao);
}

// Consultar MTRs por número
let resp = client.consultar_mtrs(&FiltroMtr {
    numeros: Some(vec!["211000520453".into()]),
    ..Default::default()
}).await?;

for m in &resp.dados {
    println!("#{} status={:?}", m.numero, m.status);
}
```

### Autenticação por CPF + senha (credenciais clássicas)

```rust
use open_mtr_core::*;

// 1. Obtém o token via credenciais (OpenMtrClassicAuth — SIGOR ignora o cnpj)
let params = AuthParams::CpfSenhaUnidade {
    cpf: "07800211428".into(),
    senha: "a34r3e".into(),
    unidade: 189788,
    cnpj: None,
};
let token = OpenMtrClassicAuth::obter_token(System::Sigor, &params).await?;

// 2. Usa o token no cliente
let client = OpenMtrClient::builder()
    .base(System::Sigor)
    .token(&token)
    .build()
    .await?;
```

## Códigos de Situação do Manifesto

| Descrição | Status |
|-----------|--------|
| Ativo / Salvo / Cadastrado | `StatusMtr::Ativo` |
| Em Transporte | `StatusMtr::EmTransporte` |
| Recebido | `StatusMtr::Recebido` |
| Cancelado | `StatusMtr::Cancelado` |
| Expirado | `StatusMtr::Expirado` |

## Métodos Específicos do SIGOR

Além dos 16 métodos universais, o SIGOR expõe endpoints próprios acessíveis via `OpenMtrClient`:

| Método | Endpoint | Descrição |
|--------|----------|-----------|
| `listar_classes_por_residuo(codigo)` | `GET /retornaListaClassePorResiduo/{cod}` | Classes compatíveis com um resíduo |
| `listar_acondicionamentos_por_estado_fisico(codigo)` | `GET /retornaListaAcondicionamentoPorEstadoFisico/{cod}` | Acondicionamentos p/ estado físico |
| `consultar_mtr_por_codigo_referencia(unidade, codigo)` | `POST /retornaManifestoSeuCodigo` | Busca MTR pelo código de referência |
| `criar_manifesto_complementar(req)` | `POST /salvarManifestoComplementar` | Cria manifesto complementar |

```rust
// Exemplo: consultar por código de referência
let mtr = client.consultar_mtr_por_codigo_referencia(189788, "abc123").await?;
```

## Diferenças de Implementação vs SINIR/MTR-PE

| Aspecto | SIGOR | SINIR | MTR-PE |
|---------|-------|-------|--------|
| Método listagens | **GET** | GET | POST |
| Autenticação token | `integration_token` direto | Refresh automático via `/token` | `integration_token` direto |
| Recebimento exige motorista/placa | ✅ Sim (opcional no domínio) | ❌ Não | ❌ Não |
| Cancelamento em lote | POST `/cancelarManifestoLote` | Individual apenas | Individual apenas |
| Listar classes por resíduo | ✅ Sim | ❌ Não | ❌ Não |
| Manifesto complementar | ✅ Sim | ❌ Não | ❌ Não |

## Gaps Conhecidos

| Gap | Impacto | Motivo |
|-----|---------|--------|
| `consulta por período` | Não implementada | SIGOR não expõe endpoint de lista por período |
| `consultar_cdfs` | Não implementado | SIGOR não possui endpoint de consulta de CDFs |
| `motorista`/`placa` no recebimento | Fallback "Motorista"/"ABC1234" se não informado | Campo opcional no `ReceberMtr` |


## Arquivos

- `core/src/openmtr/adapters/sigor.rs` — Implementação do adapter (~820 linhas)
- `wiki/sigor/WebService.md` — Documentação corrigida da API
- `core/src/openmtr/adapters/mod.rs` — Trait `MtrAdapter` com métodos específicos SIGOR
- `core/src/openmtr/system.rs` — Registro do sistema `System::Sigor`
- `core/src/openmtr/open_mtr_client.rs` — Fachada com métodos específicos SIGOR
