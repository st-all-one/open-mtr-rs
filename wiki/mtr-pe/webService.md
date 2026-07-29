# WebService do Sistema MTR-PE

**URL Base:** `https://mtr.cprh.pe.gov.br/api`

**Autenticação:** Bearer Token via `POST /gettoken` (válido por 1 hora)

---

## Índice

1. [Autenticação](#1-autenticacao)
2. [Lista de Classes](#2-lista-de-classes)
3. [Lista de Unidades](#3-lista-de-unidades)
4. [Lista de Tecnologias](#4-lista-de-tecnologias)
5. [Lista de Estado Físico](#5-lista-de-estado-fisico)
6. [Lista de Resíduo](#6-lista-de-residuo)
7. [Lista de Acondicionamento](#7-lista-de-acondicionamento)
8. [Salvar Manifesto em Lote](#8-salvar-manifesto-em-lote)
9. [Download PDF Manifesto](#9-download-pdf-manifesto)
10. [Download PDF CDF](#10-download-pdf-cdf)
11. [Emite CDF](#11-emite-cdf)
12. [Receber Manifesto em Lote](#12-receber-manifesto-em-lote)
13. [Alteração de Recebimento](#13-alteracao-de-recebimento)
14. [Aceite de Alteração](#14-aceite-de-alteracao)
15. [Cancelar Manifesto](#15-cancelar-manifesto)
16. [Retorna Manifesto por Código de Barras](#16-retorna-manifesto-por-codigo-de-barras)
17. [Lista Códigos de Barras por Período](#17-lista-codigos-de-barras-por-periodo)
18. [Lista de CDFs por Período](#18-lista-de-cdfs-por-periodo)

---

## 1. Autenticação

### `POST /gettoken`

Obtém o token JWT de acesso. O token expira em 1 hora.

**Request:**

```json
Content-Type: application/json

{
  "pessoaCodigo": 1234,
  "pessoaCnpj": "00000000000000",
  "usuarioCpf": "00000000000",
  "senha": "********"
}
```

| Campo | Tipo | Tamanho | Obrigatório | Descrição |
|-------|------|---------|-------------|-----------|
| `pessoaCodigo` | Integer | - | S | Código da unidade do usuário |
| `pessoaCnpj` | String | 14 | S | CNPJ da empresa/pessoa |
| `usuarioCpf` | String | 11 | S | CPF do usuário |
| `senha` | String | - | S | Senha de acesso |

**Response:**

```json
{
  "pessoaCodigo": 1234,
  "pessoaCnpj": "00000000000000",
  "pessoaNome": "EMPRESA_EXEMPLO LTDA",
  "usuarioCodigo": 99999,
  "usuarioNome": "USUARIO_EXEMPLO",
  "usuarioCpf": "00000000000",
  "token": "eyJhbGciOiJIUzUxMiJ9...",
  "senha": "",
  "retornoCodigo": 0,
  "retorno": "Login com Sucesso;"
}
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `token` | String | JWT Bearer token para uso nas requisições |
| `retornoCodigo` | Integer | `0` = sucesso |
| `retorno` | String | Mensagem de retorno |

**Códigos de Retorno:**

| Código | Descrição |
|--------|-----------|
| 0 | Sucesso |
| 1 | Usuário ou senha incorretos |
| 2 | Usuário temporariamente bloqueado ou cancelado |

> O token deve ser armazenado e incluído no header `Authorization: Bearer {token}` em todas as requisições subsequentes. Não consumir este endpoint para cada requisição.

---

## 2. Lista de Classes

### `POST /retornaListaClasse`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:** `{}`

**Response:**

```json
[
  {
    "tpclaCodigo": 1,
    "tpclaDescricao": "Classe I",
    "tpclaSigla": "I"
  },
  {
    "tpclaCodigo": 2,
    "tpclaDescricao": "Classe II A",
    "tpclaSigla": "IIA"
  }
]
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `tpclaCodigo` | Integer | Código da classe |
| `tpclaDescricao` | String | Descrição da classe |
| `tpclaSigla` | String | Sigla da classe |

---

## 3. Lista de Unidades

### `POST /retornaListaUnidade`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:** `{}`

**Response:**

```json
[
  {
    "tpuniCodigo": 1,
    "tpuniDescricao": "Metro Cúbico",
    "tpuniSigla": "m³"
  },
  {
    "tpuniCodigo": 2,
    "tpuniDescricao": "Litro",
    "tpuniSigla": "L"
  },
  {
    "tpuniCodigo": 3,
    "tpuniDescricao": "Quilograma",
    "tpuniSigla": "Kg"
  },
  {
    "tpuniCodigo": 4,
    "tpuniDescricao": "Tonelada",
    "tpuniSigla": "t"
  }
]
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `tpuniCodigo` | Integer | Código da unidade |
| `tpuniDescricao` | String | Descrição da unidade |
| `tpuniSigla` | String | Sigla da unidade |

---

## 4. Lista de Tecnologias

### `POST /retornaListaTecnologia`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:** `{}`

**Response:**

```json
[
  {
    "tipoCodigo": 31,
    "tipoDescricao": "Aterro Classe I"
  },
  {
    "tipoCodigo": 32,
    "tipoDescricao": "Aterro Classe IIA e IIB"
  }
]
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `tipoCodigo` | Integer | Código da tecnologia |
| `tipoDescricao` | String | Descrição da tecnologia |

---

## 5. Lista de Estado Físico

### `POST /retornaListaEstadoFisico`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:** `{}`

**Response:**

```json
[
  {
    "tpestCodigo": 1,
    "tpestDescricao": "Sólido",
    "tpestSigla": "SÓL"
  },
  {
    "tpestCodigo": 2,
    "tpestDescricao": "Líquido",
    "tpestSigla": "LÍQ"
  },
  {
    "tpestCodigo": 3,
    "tpestDescricao": "Semi-sólido",
    "tpestSigla": "S.SÓL"
  },
  {
    "tpestCodigo": 4,
    "tpestDescricao": "Gasoso",
    "tpestSigla": "GAS"
  }
]
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `tpestCodigo` | Integer | Código do estado físico |
| `tpestDescricao` | String | Descrição do estado físico |
| `tpestSigla` | String | Sigla do estado físico |

---

## 6. Lista de Resíduo

### `POST /retornaListaResiduo`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:** `{}`

**Response:**

```json
[
  {
    "tpre1Codigo": 1,
    "tpre2Codigo": 2,
    "tpre3Codigo": 879,
    "tpre1Descricao": "Resíduos da Prospecção e Exploração de Minas e Pedreiras...",
    "tpre2Descricao": "Resíduos de transformação física e química de minérios metálicos",
    "tpre3Descricao": "Lamas vermelhas da produção de alumina não abrangidas em 01 03 07",
    "tpre1Numero": "01",
    "tpre2Numero": "0103",
    "tpre3Numero": "010309"
  }
]
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `tpre1Codigo` | Integer | Código do capítulo |
| `tpre2Codigo` | Integer | Código do subcapítulo |
| `tpre3Codigo` | Integer | Código do resíduo |
| `tpre1Descricao` | String | Descrição do capítulo |
| `tpre2Descricao` | String | Descrição do subcapítulo |
| `tpre3Descricao` | String | Descrição do resíduo |
| `tpre1Numero` | String | Número do capítulo |
| `tpre2Numero` | String | Número do subcapítulo |
| `tpre3Numero` | String | Código IBAMA completo do resíduo |

> O código IBAMA do resíduo é o campo `tpre3Numero`.

---

## 7. Lista de Acondicionamento

### `POST /retornaListaAcondicionamento`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:** `{}`

**Response:**

```json
[
  {
    "tipoCodigo": 1,
    "tipoDescricao": "Tambor"
  },
  {
    "tipoCodigo": 9,
    "tipoDescricao": "E08 - Caixa de papelão"
  }
]
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `tipoCodigo` | Integer | Código do tipo de acondicionamento |
| `tipoDescricao` | String | Descrição do acondicionamento |

---

## 8. Salvar Manifesto em Lote

### `POST /salvarManifestoLote`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
{
  "manifestoJSONDtos": [
    {
      "cnpGerador": "00000000000000",
      "codUnidadeGerador": 1234,
      "cnpTransportador": "00000000000000",
      "codUnidadeTransportador": 1234,
      "cnpDestinador": "00000000000000",
      "codUnidadeDestinador": 1234,
      "cnpArmazenador": null,
      "codUnidadeArmazenador": null,
      "seuCodigoReferencia": "123456",
      "manifObservacao": "Observações do manifesto",
      "manifGeradorNomeResponsavel": "NOME DO RESPONSAVEL",
      "manifGeradorCargoResponsavel": "CARGO",
      "manifTransportadorNomeMotorista": "NOME DO MOTORISTA",
      "manifTransportadorPlacaVeiculo": "ABC-1234",
      "manifTransportadorDataExpedicao": "20260729",
      "itemManifestoJSONs": [
        {
          "quantidade": 1.00,
          "residuo": "010101",
          "codigoAcondicionamento": 1,
          "codigoClasse": 1,
          "codigoTecnologia": 31,
          "codigoTipoEstado": 1,
          "codigoUnidade": 1,
          "manifestoItemObservacao": "",
          "manifestoItemCodInterno": "",
          "manifestoItemCodInternoDestinador": "",
          "tipoDensidadeValor": 1,
          "tipoDensidadeUnidade": 1,
          "classeDeRisco": null,
          "numeroONU": null,
          "nomeEmbarque": null,
          "grupoEmbalagem": null
        }
      ]
    }
  ]
}
```

**Campos do Manifesto (`manifestoJSONDtos[]`):**

| Campo | Tipo | Tamanho | Obrigatório | Descrição |
|-------|------|---------|-------------|-----------|
| `cnpGerador` | String | 14 | S | CNPJ do gerador |
| `codUnidadeGerador` | Integer | - | N | Código da unidade do gerador |
| `cnpTransportador` | String | 14 | S | CNPJ do transportador |
| `codUnidadeTransportador` | Integer | - | N | Código da unidade do transportador |
| `cnpDestinador` | String | 14 | S | CNPJ do destinador |
| `codUnidadeDestinador` | Integer | - | N | Código da unidade do destinador |
| `cnpArmazenador` | String | 14 | N | CNPJ do armazenador temporário |
| `codUnidadeArmazenador` | Integer | - | N | Código da unidade do armazenador |
| `seuCodigoReferencia` | Integer | - | N | Código de referência interno (deve ser número, **não** string) |
| `manifObservacao` | String | 4000 | N | Observações do manifesto |
| `manifGeradorNomeResponsavel` | String | 300 | S | Nome do responsável pela emissão |
| `manifGeradorCargoResponsavel` | String | 100 | S | Cargo do responsável pela emissão |
| `manifTransportadorNomeMotorista` | String | 300 | N | Nome do motorista |
| `manifTransportadorPlacaVeiculo` | String | 10 | N | Placa do veículo |
| `manifTransportadorDataExpedicao` | String | 8 | N | Data de expedição (yyyyMMdd) |
| `itemManifestoJSONs` | Array | - | S | Itens do manifesto (resíduos) |

**Campos do Item (`itemManifestoJSONs[]`):**

| Campo | Tipo | Tamanho | Obrigatório | Descrição |
|-------|------|---------|-------------|-----------|
| `quantidade` | Number | - | S | Quantidade do resíduo |
| `residuo` | String | 20 | S | Código IBAMA do resíduo |
| `codigoAcondicionamento` | Integer | - | S | Código do tipo de acondicionamento |
| `codigoClasse` | Integer | - | S | Código da classe do resíduo |
| `codigoTecnologia` | Integer | - | S | Código da tecnologia de destinação |
| `codigoTipoEstado` | Integer | - | S | Código do estado físico |
| `codigoUnidade` | Integer | - | S | Código da unidade de medida |
| `manifestoItemObservacao` | String | - | N | Descrição interna do gerador |
| `manifestoItemCodInterno` | String | - | N | Código interno do gerador |
| `manifestoItemCodInternoDestinador` | String | - | N | Código interno do destinador |
| `tipoDensidadeValor` | Integer | - | S | Valor da densidade (1 para não litro) |
| `tipoDensidadeUnidade` | Integer | - | S | Unidade da densidade (1=padrão, 2=litro) |
| `classeDeRisco` | String | 1 | N | Classe de risco (1 a 9) |
| `numeroONU` | String | 4 | N | Número ONU (0000 a 9999) |
| `nomeEmbarque` | String | - | N | Nome apropriado para embarque |
| `grupoEmbalagem` | String | - | N | Grupo de embalagem |

> `tipoDensidadeValor` e `tipoDensidadeUnidade` devem ser 1 para todas as unidades, exceto Litro, onde `tipoDensidadeUnidade` deve ser 2.

**Response (Sucesso):**

```json
{
  "manifestoJSONDtos": [
    {
"manifestoCodigo": 1111111111,
      "seuCodigoReferencia": "123456",
      "retornoCodigo": 0,
      "cnpGerador": "00000000000000",
      "codUnidadeGerador": 1234,
      "cnpTransportador": "00000000000000",
      "codUnidadeTransportador": 1234,
      "cnpDestinador": "00000000000000",
      "codUnidadeDestinador": 1234,
      "retorno": "MTR Salvo",
      "itemManifestoJSONs": [
        {
          "codigoSequencial": 1,
          "quantidade": 1.0,
          "residuo": "010101",
          "codigoAcondicionamento": 1,
          "codigoClasse": 1,
          "codigoTecnologia": 31,
          "codigoTipoEstado": 1,
          "codigoUnidade": 1,
          "tipoDensidadeValor": "1",
          "tipoDensidadeUnidade": "1"
        }
      ]
    }
  ]
}
```

**Response (Erro):**

```json
{
  "manifestoJSONDtos": [
    {
      "retornoCodigo": 902,
      "retorno": "O Transportador possui mais de uma unidade com o mesmo CNPJ ou está incorreto!"
    }
  ]
}
```

**Códigos de Retorno:**

| Código | Descrição |
|--------|-----------|
| 0 | MTR Salvo |
| 900 | Erro ao consultar, usuário e/ou senha inexistente no sistema MTR |
| 901 | Dados do manifesto inválidos - Cnpj do token informado não é de Gerador |
| 902 | Transportador/Destinador com mais de uma unidade ou CNPJ incorreto |
| 905 | Responsável não informado no MTR |
| 906 | Falta informar os dados do manifesto |
| 907 | Data de Transporte não pode ser inferior a data de Emissão |
| 908 | Falta informar os dados dos resíduos |
| 910-925 | Erros de validação de resíduo (acondicionamento, classe, estado físico, etc.) |
| 999 | Erro genérico |

---

## 9. Download PDF Manifesto

### `POST /buscaPdfManifestoPorCodigoBarras/{CODIGO_BARRA}`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:** `{}`

O código de barras do manifesto é composto por:
- Número do Manifesto (10 posições)
- Código da Unidade do Gerador (8 posições)
- Código da Unidade do Transportador (8 posições)
- Código da Unidade do Destinador (8 posições)

**Response:** Arquivo PDF do manifesto.

**Códigos de Retorno:**

| Código | Descrição |
|--------|-----------|
| 901 | Dados do manifesto inválidos |
| 902 | Manifesto Inválido ou Não encontrado |
| 903 | Código de Barras inválido |

---

## 10. Download PDF CDF

### `POST /buscaPdfCdf/{NUMERO_CDF}`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:** `{}`

**Response:** Arquivo PDF do CDF.

**Códigos de Retorno:**

| Código | Descrição |
|--------|-----------|
| 901 | Certificado Inválido ou Não encontrado |

---

## 11. Emite CDF

### `POST /emiteCDF`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
{
  "dataInicial": "20260701",
  "dataFinal": "20260729",
  "responsavel": "123",
  "txtObservacoes": "",
  "listaGerador": ["00000000000000"],
  "listaMtr": []
}
```

| Campo | Tipo | Tamanho | Obrigatório | Descrição |
|-------|------|---------|-------------|-----------|
| `dataInicial` | String | 8 | S | Data inicial (yyyyMMdd) |
| `dataFinal` | String | 8 | S | Data final (yyyyMMdd) |
| `responsavel` | String | 255 | S | Identificador do responsável técnico |
| `txtObservacoes` | String | 255 | N | Observações |
| `listaGerador` | Array | - | N | Lista de CNPJs dos geradores |
| `listaMtr` | Array | - | N | Lista de números de MTR |

> Se ambas as listas forem informadas, `listaGerador` é ignorada.

**Response:**

```json
{
  "dataInicial": "01/07/2026",
  "responsavel": "NOME",
  "dataFinal": "29/07/2026",
  "txtObservacoes": null,
  "listaGerador": ["00000000000000"],
  "listaMtr": null,
  "retornoCodigo": 0,
  "retorno": "Sucesso",
  "listaCertificadoJSON": {
    "codigo": 77777,
    "cnpGerador": "00000000000000",
    "data": "29/07/2026",
    "itemManifestoJSONs": []
  }
}
```

**Códigos de Retorno:**

| Código | Descrição |
|--------|-----------|
| 0 | Sucesso |
| 900 | Cnpj do token informado não é de Destinador |
| 901 | Responsável Técnico não encontrado |
| 902 | Nenhum MTR encontrado para o período |
| 999 | Erro genérico |

---

## 12. Receber Manifesto em Lote

### `POST /receberManifestoLote`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
{
  "manifestoRecebimentoJSONs": [
    {
      "manifestoCodigo": "1111111111",
      "cnpGerador": "00000000000000",
      "cnpTransportador": "00000000000000",
      "recebimentoMtrResponsavel": "RESPONSAVEL_EXEMPLO",
      "recebimentoMtrCargo": "CARGO_EXEMPLO",
      "recebimentoMtrData": "20260729",
      "transporteMtrData": "20260729",
      "recebimentoMtrObs": "",
      "nomeMotorista": "Motorista",
      "placaVeiculo": "ABC-1234",
      "itemManifestoRecebimentoJSONs": [
        {
          "codigoSequencial": 1,
          "justificativa": null,
          "codigoInterno": null,
          "qtdRecebida": 1.11,
          "residuo": "010101",
          "codigoTecnologia": 31
        }
      ]
    }
  ]
}
```

**Campos do Item de Recebimento:**

| Campo | Tipo | Obrigatório | Descrição |
|-------|------|-------------|-----------|
| `codigoSequencial` | Integer | N | Sequencial do item |
| `qtdRecebida` | Number | S | Quantidade recebida |
| `residuo` | String | S | Código IBAMA do resíduo |
| `codigoTecnologia` | Integer | S | Código da tecnologia de destinação |
| `justificativa` | String | N | Justificativa (obrigatória se qtdRecebida = 0) |

**Response:**

```json
{
  "manifestoRecebimentoJSONs": [
    {
      "manifestoCodigo": 1111111111,
      "retornoCodigo": 0,
      "retorno": "MTR Recebido",
      "itemManifestoRecebimentoJSONs": [
        {
          "codigoSequencial": 1,
          "qtdRecebida": 1.11,
          "residuo": "010101",
          "codigoTecnologia": 31
        }
      ]
    }
  ]
}
```

**Códigos de Retorno:**

| Código | Descrição |
|--------|-----------|
| 0 | MTR Recebido |
| 901 | Manifesto em situação diferente de salvo |
| 902 | Manifesto Inválido ou Não encontrado |
| 905 | Responsável recebimento não encontrado |
| 907 | Dados do manifesto inválidos / Destinador Inválido |
| 908-914 | Erros de validação de recebimento |
| 999 | Lista de itens está vazia |

---

## 13. Alteração de Recebimento

### `POST /alteraRecebimento`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
{
  "manifestoAlterarJSONs": [
    {
      "manifestoCodigo": 2222222222,
      "residuo": "010101",
      "residuoNovo": "010101",
      "qtdRecebidaNovo": 1,
      "codigoClasseNovo": 1,
      "codigoTecnologiaNovo": 1,
      "codigoUnidadeNovo": 1
    }
  ]
}
```

| Campo | Tipo | Obrigatório | Descrição |
|-------|------|-------------|-----------|
| `manifestoCodigo` | Integer | S | Código do Manifesto |
| `residuo` | String | S | Código do resíduo atual |
| `residuoNovo` | String | N | Novo código do resíduo |
| `qtdRecebidaNovo` | Number | N | Nova quantidade recebida |
| `codigoClasseNovo` | Integer | N | Nova classe |
| `codigoTecnologiaNovo` | Integer | N | Nova tecnologia |
| `codigoUnidadeNovo` | Integer | N | Nova unidade |

**Response:**

```json
{
  "manifestoAlterarJSONs": [
    {
      "manifestoCodigo": 2222222222,
      "retornoCodigo": 0,
      "retorno": "Recebimento alterado, aguarda autorização do Gerador"
    }
  ]
}
```

**Códigos de Retorno:**

| Código | Descrição |
|--------|-----------|
| 0 | Recebimento alterado, aguarda autorização do Gerador |
| 901 | Destinador Informado diferente do informado no token |
| 902 | Manifesto Inválido ou Não encontrado |
| 903 | Destinador Inválido para este manifesto |
| 904 | Manifesto em situação diferente de recebido |
| 905 | Solicitação de alteração já registrada |
| 906-910 | Erros de validação |
| 999 | Erro genérico |

---

## 14. Aceite de Alteração

### `POST /aceiteAlteracaoRecebimento`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
{
  "codigoManifesto": 4444444444,
  "aceite": "S"
}
```

| Campo | Tipo | Obrigatório | Descrição |
|-------|------|-------------|-----------|
| `codigoManifesto` | Integer | S | Código do Manifesto |
| `aceite` | String | S | `S` para aceitar, `N` para recusar |

**Response:**

```json
{
  "aceite": "S",
  "retornoCodigo": 0,
  "retorno": "Solicitacao de alteracao do MTR n 4444444444 Aceita com sucesso!"
}
```

**Códigos de Retorno:**

| Código | Descrição |
|--------|-----------|
| 0 | Solicitação aceita/recusada com sucesso |
| 900 | Usuário ou senha inválidos |
| 901 | Gerador com mais de uma unidade cadastrada |
| 902 | Solicitação de alteração não encontrada |
| 903 | Manifesto não encontrado ou não pertence ao gerador |
| 904 | Opção inválida |

---

## 15. Cancelar Manifesto

### `POST /cancelarManifesto`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
{
  "manifestoCodigo": "123456",
  "justificativa": "teste"
}
```

| Campo | Tipo | Obrigatório | Descrição |
|-------|------|-------------|-----------|
| `manifestoCodigo` | String | S | Código do Manifesto |
| `justificativa` | String | S | Justificativa do cancelamento |

**Response:**

```json
{
  "manifestoCodigo": 123456,
  "justificativa": "teste",
  "retornoCodigo": 0,
  "retorno": "Sucesso - Manifesto Cancelado"
}
```

**Códigos de Retorno:**

| Código | Descrição |
|--------|-----------|
| 0 | Sucesso - Manifesto Cancelado |
| 902 | Manifesto Inválido ou não encontrado |
| 903 | Manifesto não é do gerador enviado no token |
| 909 | Somente o gerador pode solicitar o cancelamento |
| 910 | Situação do manifesto inválida para cancelamento |
| 911 | Justificativa para cancelamento não informada |

---

## 16. Retorna Manifesto por Código de Barras

### `POST /retornaManifesto/{CODIGO_BARRA}`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:** `{}`

O código de barras do manifesto é composto por:
- Número do Manifesto (10 posições)
- Código da Unidade do Gerador (8 posições)
- Código da Unidade do Transportador (8 posições)
- Código da Unidade do Destinador (8 posições)

**Response:**

```json
{
  "manifestoCodigo": 3333333333,
  "seuCodigoReferencia": null,
  "retornoCodigo": 0,
  "cnpGerador": "00000000000000",
  "codUnidadeGerador": 1234,
  "cnpTransportador": "00000000000000",
  "codUnidadeTransportador": 1234,
  "cnpDestinador": "00000000000000",
  "codUnidadeDestinador": 1234,
  "situacaoManifestoCodigo": 4,
  "manifData": null,
  "manifDataExpedicao": null,
  "manifObservacao": "",
  "manifGeradorNomeResponsavel": "RESPONSAVEL",
  "manifGeradorCargoResponsavel": "CARGO",
  "manifTransportadorNomeMotorista": "",
  "manifTransportadorPlacaVeiculo": "",
  "retorno": "Sucesso",
  "cdfCodigo": null,
  "itemManifestoJSONs": [
    {
      "codigoSequencial": 1,
      "quantidade": 3.0,
      "residuo": "010102",
      "codigoAcondicionamento": 7,
      "codigoClasse": 2,
      "codigoTecnologia": 31,
      "codigoTipoEstado": 1,
      "codigoUnidade": 4,
      "tipoDensidadeValor": null,
      "tipoDensidadeUnidade": null,
      "numeroONU": "3234",
      "classeDeRisco": "3",
      "nomeEmbarque": null,
      "grupoEmbalagem": null
    }
  ]
}
```

**Campos adicionais:** `situacaoManifestoCodigo`:

| Código | Situação |
|--------|----------|
| 1 | Salvo |
| 3 | MTR Recebido |
| 4 | Cancelado |
| 9 | Em Armazenamento Temporário |

**Códigos de Retorno:**

| Código | Descrição |
|--------|-----------|
| 0 | Sucesso |
| 901 | Código de Barras Inválido |
| 902 | Manifesto Inválido ou não encontrado |
| 903 | Erro Não Identificado |

---

## 17. Lista Códigos de Barras por Período

### `POST /retornaListaCodigoBarasManifesto/{dataInicio}/{dataFim}`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Parâmetros de URL:**
- `dataInicio` — Data inicial no formato `yyyyMMdd`
- `dataFim` — Data final no formato `yyyyMMdd`

**Response:**

```json
{
  "retornoCodigo": 0,
  "retorno": "Sucesso.",
  "codigos": [
    "2114014590000035810000091700001234",
    "2114015467000011590000091700001234"
  ]
}
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `codigos` | Array\[String\] | Lista de códigos de barras dos manifestos |

**Códigos de Retorno:**

| Código | Descrição |
|--------|-----------|
| 0 | Sucesso |
| 901 | Nenhum manifesto encontrado |

---

## 18. Lista de CDFs por Período

### `POST /consultaListaCdf`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
{
  "cdfDataInicial": "01/07/2026",
  "cdfDataFinal": "29/07/2026",
  "cdfGeradorCodigo": null,
  "cdfDestinadorCodigo": null
}
```

| Campo | Tipo | Obrigatório | Descrição |
|-------|------|-------------|-----------|
| `cdfDataInicial` | String | S | Data inicial (dd/MM/yyyy) |
| `cdfDataFinal` | String | S | Data final (dd/MM/yyyy) |
| `cdfGeradorCodigo` | String | N | Filtro por código do gerador |
| `cdfDestinadorCodigo` | String | N | Filtro por código do destinador |

**Response:**

```json
{
  "cdfDataInicial": "01/07/2026",
  "cdfDataFinal": "29/07/2026",
  "retornoCodigo": 0,
  "retorno": "Sucesso.",
  "listaCdfDTO": [
    {
      "cdfCodigo": "888888",
      "cdfDataEmissao": 1558013347775,
      "cdfDataInicial": 1557975600000,
      "cdfDataFinal": 1557975600000,
      "cdfDestinadorCodigo": null,
      "cdfGeradorCodigo": null,
      "listaMtr": []
    }
  ]
}
```

**Códigos de Retorno:**

| Código | Descrição |
|--------|-----------|
| 0 | Sucesso |
| 901 | Nenhum CDF encontrado |
| 902 | Dados inválidos |
| 903 | Período máximo de 1 mês excedido |

---

## Tabela de Códigos de Retorno Gerais

| Código | Descrição |
|--------|-----------|
| 0 | Sucesso |
| 1 | Usuário ou senha incorretos |
| 2 | Usuário bloqueado ou cancelado |
| 900 | Erro de autenticação/credenciais |
| 901 | Dados inválidos |
| 902 | Registro não encontrado |
| 903 | Erro não identificado |
| 904 | Conflito/opção inválida |
| 905-930 | Erros de validação de negócio |
| 999 | Erro genérico |
