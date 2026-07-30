# WebService SIGOR-MTR (CETESB/SP)

**Sistema:** SIGOR — Sistema Estadual de Gerenciamento Online de Resíduos Sólidos, Módulo MTR
**Órgão:** CETESB — São Paulo
**Base URL (produção):** `https://mtrr.cetesb.sp.gov.br/apiws/rest`
**Base URL (homologação):** `https://mtrr-hom.cetesb.sp.gov.br/apiws/rest`
**Autenticação:** Bearer Token via `POST /gettoken` (sem refresh automático)

---

## Índice

1. [Autenticação](#1-autenticacao)
2. [Lista de Classes](#2-lista-de-classes)
3. [Lista de Unidades](#3-lista-de-unidades)
4. [Lista de Tratamentos](#4-lista-de-tratamentos)
5. [Lista de Estados Físicos](#5-lista-de-estados-fisicos)
6. [Lista de Acondicionamentos](#6-lista-de-acondicionamentos)
7. [Lista de Resíduos](#7-lista-de-residuos)
8. [Lista de Classes por Resíduo](#8-lista-de-classes-por-residuo)
9. [Lista de Acondicionamentos por Estado Físico](#9-lista-de-acondicionamentos-por-estado-fisico)
10. [Gerar Manifesto em Lote](#10-gerar-manifesto-em-lote)
11. [Download do Manifesto](#11-download-do-manifesto)
12. [Cancelar Manifesto](#12-cancelar-manifesto)
13. [Retorna Manifesto](#13-retorna-manifesto)
14. [Receber Manifesto em Lote](#14-receber-manifesto-em-lote)
15. [Gerar CDF](#15-gerar-cdf)
16. [Download do CDF](#16-download-do-cdf)
17. [Alteração de Recebimento](#17-alteracao-de-recebimento)
18. [Aceite de Alteração](#18-aceite-de-alteracao)
19. [Cancelar Manifesto em Lote](#19-cancelar-manifesto-em-lote)
20. [Consulta MTR por Código de Referência](#20-consulta-mtr-por-codigo-de-referencia)
21. [Gerar Manifesto Complementar](#21-gerar-manifesto-complementar)

---

## 1. Autenticação

### `POST /gettoken`

Obtém o token JWT de acesso. O token tem validade definida pelo servidor; não há endpoint de refresh — reautentique quando expirar.

**Request:**

```json
{
  "cpfCnpj": "07800211428",
  "senha": "********",
  "unidade": "189788"
}
```

| Campo | Tipo | Tamanho | Obrigatório | Descrição |
|-------|------|---------|-------------|-----------|
| `cpfCnpj` | String | 11 ou 14 | S | CPF (11 dígitos) ou CNPJ (14 dígitos) do usuário |
| `senha` | String | - | S | Senha de acesso |
| `unidade` | String | - | S | Código da unidade do empreendimento |

> O campo `cpfCnpj` deve conter o **CPF do usuário** para autenticação bem-sucedida. O CNPJ da empresa não funciona neste campo.

**Response (sucesso):**

```json
{
  "mensagem": "Sucesso!",
  "objetoResposta": "Bearer eyJhbGciOiJIUzUxMiJ9...",
  "erro": false
}
```

O token JWT está no campo `objetoResposta` com o prefixo `Bearer `. Extraia o valor após o prefixo e inclua no header `Authorization: Bearer {token}` em todas as requisições subsequentes.

**Response (erro):**

```json
{
  "mensagem": "ERRO 007: Autenticação inválida.",
  "objetoResposta": null,
  "erro": true
}
```

**Códigos de Retorno:**

| Código | Mensagem |
|--------|----------|
| 000 | Sucesso |
| 001 | Problema na autenticação! (cpfCnpj inválido) |
| 002 | parâmetro cpfCnpj deve ser informado e não pode ser vazio! |
| 003 | parâmetro senha deve ser informado e não pode ser vazio! |
| 004 | parâmetro unidade deve ser informado e não pode ser vazio! |
| 005 | parâmetro cpfCnpj não é válido! |
| 006 | parâmetro unidade deve conter apenas números! |
| 007 | Autenticação inválida, verifique parâmetros |

---

## 2. Lista de Classes

### `GET /retornaListaClasse`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Response:**

```json
{
  "mensagem": null,
  "objetoResposta": [
    { "claCodigo": 1, "claDescricao": "CLASSE I" },
    { "claCodigo": 11, "claDescricao": "CLASSE A (RCC)" },
    { "claCodigo": 12, "claDescricao": "CLASSE B (RCC)" },
    { "claCodigo": 13, "claDescricao": "CLASSE C (RCC)" },
    { "claCodigo": 14, "claDescricao": "CLASSE D (RCC)" },
    { "claCodigo": 21, "claDescricao": "GRUPO A1 (RSS)" },
    ...
  ],
  "erro": false
}
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `claCodigo` | Integer | Código da classe |
| `claDescricao` | String | Descrição da classe |

---

## 3. Lista de Unidades

### `GET /retornaListaUnidade`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Response:**

```json
{
  "mensagem": null,
  "objetoResposta": [
    { "uniCodigo": 21, "uniDescricao": "Litro", "uniSigla": "L" },
    { "uniCodigo": 20, "uniDescricao": "M³", "uniSigla": "M³" },
    { "uniCodigo": 2, "uniDescricao": "Quilograma", "uniSigla": "KG" },
    { "uniCodigo": 3, "uniDescricao": "Tonelada", "uniSigla": "TON" },
    { "uniCodigo": 4, "uniDescricao": "Unidade", "uniSigla": "UN" },
    ...
  ],
  "erro": false
}
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `uniCodigo` | Integer | Código da unidade |
| `uniDescricao` | String | Descrição da unidade |
| `uniSigla` | String | Sigla da unidade |

---

## 4. Lista de Tratamentos

### `GET /retornaListaTratamento`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Response:**

```json
{
  "mensagem": null,
  "objetoResposta": [
    { "traCodigo": 51, "traDescricao": "Aterro de Reservação - RCC" },
    { "traCodigo": 4, "traDescricao": "Aterro Resíduos Classe I" },
    { "traCodigo": 17, "traDescricao": "Aterro Resíduos Classes IIA e IIB" },
    { "traCodigo": 52, "traDescricao": "Aterro Sanitário - RSU" },
    ...
  ],
  "erro": false
}
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `traCodigo` | Integer | Código do tratamento |
| `traDescricao` | String | Descrição do tratamento |

---

## 5. Lista de Estados Físicos

### `GET /retornaListaEstadoFisico`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Response:**

```json
{
  "mensagem": null,
  "objetoResposta": [
    { "tieCodigo": 1, "tieDescricao": "SEMISSÓLIDO", "tieCodigoReferencia": 3 },
    { "tieCodigo": 2, "tieDescricao": "LIQUIDO", "tieCodigoReferencia": 2 },
    { "tieCodigo": 3, "tieDescricao": "GASOSO", "tieCodigoReferencia": 4 },
    { "tieCodigo": 4, "tieDescricao": "SÓLIDO", "tieCodigoReferencia": 1 },
    ...
  ],
  "erro": false
}
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `tieCodigo` | Integer | Código do estado físico |
| `tieDescricao` | String | Descrição do estado físico |
| `tieCodigoReferencia` | Integer | Código de referência (mapeamento SINIR) |

---

## 6. Lista de Acondicionamentos

### `GET /retornaListaAcondicionamento`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Response:**

```json
{
  "mensagem": null,
  "objetoResposta": [
    { "tiaCodigo": 2, "tiaDescricao": "SACO PLÁSTICO", "tiaCodigoReferencia": 14 },
    { "tiaCodigo": 3, "tiaDescricao": "CONTEINER", "tiaCodigoReferencia": 13 },
    { "tiaCodigo": 4, "tiaDescricao": "CAÇAMBA ABERTA", "tiaCodigoReferencia": 2 },
    ...
  ],
  "erro": false
}
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `tiaCodigo` | Integer | Código do acondicionamento |
| `tiaDescricao` | String | Descrição do acondicionamento |
| `tiaCodigoReferencia` | Integer | Código de referência (mapeamento SINIR) |

---

## 7. Lista de Resíduos

### `GET /retornaListaResiduo`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Response:**

```json
{
  "mensagem": null,
  "objetoResposta": [
    { "resCodigoIbama": "070109(*)", "resDescricao": "Absorventes usados e tortas de filtro halogenados" },
    { "resCodigoIbama": "070209(*)", "resDescricao": "Absorventes usados e tortas de filtro halogenados" },
    ...
  ],
  "erro": false
}
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `resCodigoIbama` | String | Código IBAMA do resíduo (com `(*)` se perigoso) |
| `resDescricao` | String | Descrição do resíduo |

> **Atenção:** O campo retornado é `resCodigoIbama` (com "I" maiúsculo), não `resCodigolbama`.

---

## 8. Lista de Classes por Resíduo

### `GET /retornaListaClassePorResiduo/{resCodigoIbama}`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

Retorna as classes possíveis para um determinado resíduo.

**Exemplo:** `GET /retornaListaClassePorResiduo/180101(*)`

**Response:**

```json
{
  "mensagem": null,
  "objetoResposta": [
    { "claCodigo": 1, "claDescricao": "CLASSE I" },
    { "claCodigo": 21, "claDescricao": "GRUPO A1 (RSS)" }
  ],
  "erro": false
}
```

---

## 9. Lista de Acondicionamentos por Estado Físico

### `GET /retornaListaAcondicionamentoPorEstadoFisico/{tieCodigo}`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

Retorna os acondicionamentos compatíveis com um determinado estado físico.

**Exemplo:** `GET /retornaListaAcondicionamentoPorEstadoFisico/1`

**Response:**

```json
{
  "mensagem": null,
  "objetoResposta": [
    { "tiaCodigo": 14, "tiaDescricao": "BOMBONA PLÁSTICA", "tiaCodigoReferencia": 0 },
    { "tiaCodigo": 4, "tiaDescricao": "CAÇAMBA ABERTA", "tiaCodigoReferencia": 2 },
    ...
  ],
  "erro": false
}
```

> **Nota:** O nome do endpoint **não** possui acento: `EstadoFisico` e não `EstadoFísico`.

---

## 10. Gerar Manifesto em Lote

### `POST /salvarManifestoLote`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
{
  "nomeResponsavel": "nome responsavel",
  "seuCodigo": "abc123",
  "dataExpedicao": 1622592000000,
  "nomeMotorista": "motorista",
  "placaVeiculo": "abc1234",
  "observacoes": "obs",
  "transportador": {
    "unidade": 99999,
    "cpfCnpj": "9999999999999999"
  },
  "destinador": {
    "unidade": 99999,
    "cpfCnpj": "9999999999999"
  },
  "armazenadorTemporario": {
    "unidade": 99999,
    "cpfCnpj": "9999999999999"
  },
  "listaManifestoResiduos": [
    {
      "marQuantidade": 10.0,
      "resCodigoIbama": "010101",
      "uniCodigo": 3,
      "traCodigo": 4,
      "tieCodigo": 4,
      "tiaCodigo": 26,
      "claCodigo": 42,
      "marDensidade": 1,
      "marNumeroONU": "5255",
      "marClasseRisco": "568",
      "marNomeEmbarque": "nome embarque",
      "greCodigo": 1
    }
  ]
}
```

| Parâmetro | Tipo | Obrigatório | Descrição |
|-----------|------|-------------|-----------|
| `nomeResponsavel` | String | S | Nome do responsável pela emissão |
| `seuCodigo` | String | N | Código de referência interno (máx. 30 caracteres, único) |
| `dataExpedicao` | Long | N | Timestamp ms da data de expedição |
| `nomeMotorista` | String | N | Nome do motorista |
| `placaVeiculo` | String | N | Placa do veículo (sem hífen, máx. 7 caracteres) |
| `observacoes` | String | N | Observações |
| `transportador.unidade` | Integer | S | Código da unidade do transportador |
| `transportador.cpfCnpj` | String | S | CPF/CNPJ do transportador |
| `destinador.unidade` | Integer | S | Código da unidade do destinador |
| `destinador.cpfCnpj` | String | S | CPF/CNPJ do destinador |
| `armazenadorTemporario.unidade` | Integer | N | Código da unidade do armazenador |
| `armazenadorTemporario.cpfCnpj` | String | N | CPF/CNPJ do armazenador |

**Campos de `listaManifestoResiduos[]`:**

| Parâmetro | Tipo | Obrigatório | Descrição |
|-----------|------|-------------|-----------|
| `marQuantidade` | Double | S | Quantidade do resíduo |
| `resCodigoIbama` | String | S | Código IBAMA do resíduo |
| `uniCodigo` | Integer | S | Código da unidade |
| `traCodigo` | Integer | S | Código do tratamento |
| `tieCodigo` | Integer | S | Código do estado físico |
| `tiaCodigo` | Integer | S | Código do acondicionamento |
| `claCodigo` | Integer | S | Código da classe |
| `marDensidade` | Double | N | Densidade (obrigatório se unidade for Litro ou M³) |
| `marNumeroONU` | String | N | Número ONU (para resíduos perigosos) |
| `marClasseRisco` | String | N | Classe de risco (para resíduos perigosos) |
| `marNomeEmbarque` | String | N | Nome de embarque (para resíduos perigosos) |
| `greCodigo` | Integer | N | Grupo de embalagem (1=I, 2=II, 3=III, 4=N/A) |

**Observações:**

- `seuCodigo` deve ser único por gerador; usado para consulta via API 20.
- `marDensidade` obrigatório quando `uniCodigo` for Litro (21) ou M³ (20).
- Campos de periculosidade (`marNumeroONU`, `marClasseRisco`, `marNomeEmbarque`, `greCodigo`) são obrigatórios para resíduos perigosos (com `(*)` no código IBAMA) ou resíduos de saúde (Grupo A, B, C, D, E).
- Armazenador Temporário: omitir o campo se não houver.
- Placa do veículo: sem hífen.

**Response:**

```json
{
  "restResponseValido": true,
  "restResponseMensagem": "Validação de campos efetuada com Sucesso!",
  "codigoGerado": 0,
  "manifestoCodigoEstadual": null,
  "manifestoNumeroEstadual": null,
  "manifestoNumeroNacional": 211000520453,
  "nomeResponsavel": "nome responsavel",
  "transportador": { "restResponseValido": true, ... },
  "destinador": { "restResponseValido": true, ... },
  "listaManifestoResiduos": [ ... ],
  "erroNacional": false,
  "mensagemErroNacional": null
}
```

**Códigos de Retorno:**

| Código | Descrição |
|--------|-----------|
| 000 | Manifesto Gerado |
| 001 | Problema ao recuperar o usuário |
| 002 | Token não é de um Gerador |
| 003 | JSON inválido ou vazio |
| 004 | nomeResponsavel não informado |
| 005 | Data de expedição anterior à data atual |
| 006 | listaManifestoResiduos vazia |
| 007 | MTR de AT permite apenas um resíduo |
| 999 | Erro(s) encontrado(s) |

---

## 11. Download do Manifesto

### `POST /downloadManifesto/{manNumero}`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request body:** `{}`

O parâmetro `manNumero` é o número do MTR (path parameter).

**Response:** Arquivo PDF do manifesto (se HTTP 200 e não for mensagem de erro).

**Possíveis respostas de erro (texto simples):**
1. Manifesto não encontrado
2. WS Não disponível no momento.
3. O Token não pertence ao Gerador/Destinador/Transportador/Armazenador Temporário.

---

## 12. Cancelar Manifesto

### `POST /cancelarManifesto`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
{
  "manNumero": "999999999999",
  "justificativa": "teste de api"
}
```

| Parâmetro | Tipo | Obrigatório | Descrição |
|-----------|------|-------------|-----------|
| `manNumero` | String | S | Número do manifesto |
| `justificativa` | String | S | Justificativa do cancelamento |

**Response (sucesso):**

```json
{
  "mensagem": "Manifesto cancelado com sucesso",
  "objetoResposta": null,
  "erro": false
}
```

> Apenas o Gerador pode cancelar um Manifesto. Para cancelamento em lote, use a API 19.

---

## 13. Retorna Manifesto

### `GET /retornaManifesto/{manNumero}`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Response:**

```json
{
  "mensagem": null,
  "objetoResposta": {
    "manNumero": "999999999999",
    "manData": 1614049200000,
    "manResponsavel": "nome do responsavel",
    "manDataExpedicao": 1614049200000,
    "manNomeMotorista": "motorista",
    "manPlacaVeiculo": "ASD2234",
    "manObservacao": "",
    "manJustificativaCancelamento": "",
    "estado": { "estCodigo": 24, "estAbreviacao": "SC" },
    "parceiroGerador": { "parCodigo": 99999, "parDescricao": "...", "parCnpj": "..." },
    "parceiroTransportador": { ... },
    "parceiroDestinador": { ... },
    "parceiroArmazenadorTemporario": { ... },
    "situacaoManifesto": { "simCodigo": 3, "simDescricao": "Recebido", "simOrdem": 3 },
    "listaManifestoResiduo": [
      {
        "residuo": { "resCodigo": 559, "resCodigoIbama": "180101(*)", "resDescricao": "..." },
        "unidade": { "uniCodigo": 3, "uniDescricao": "Tonelada", "uniSigla": "TON" },
        "tratamento": { "traCodigo": 4, "traDescricao": "Aterro Resíduos Classe I" },
        "tipoEstado": { "tieCodigo": 1, "tieDescricao": "SEMISSÓLIDO" },
        "tipoAcondicionamento": { "tiaCodigo": 21, "tiaDescricao": "CAIXA" },
        "classe": { "claCodigo": 1, "claDescricao": "CLASSE I" },
        "marQuantidade": 0.05,
        "marQuantidadeRecebida": null,
        "marDensidade": null,
        "marNumeroONU": null,
        "marClasseRisco": null,
        "marNomeEmbarque": "nome embarque",
        "grupoEmbalagem": { "greCodigo": 1, "greDescricao": "I" }
      }
    ],
    "manNumeroEstadual": null,
    "cdfCodigo": null,
    "manResponsavelRecebimento": null
  },
  "erro": false
}
```

**Campos do objetoResposta:**

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `manNumero` | String | Número do MTR |
| `manData` | Long | Timestamp ms da data de criação |
| `situacaoManifesto.simDescricao` | String | Situação (Ativo, Recebido, Cancelado) |
| `parceiroGerador.parCnpj` | String | CNPJ do gerador |
| `listaManifestoResiduo[].residuo.resCodigoIbama` | String | Código IBAMA do resíduo |

---

## 14. Receber Manifesto em Lote

### `POST /receberManifestoLote`

> **Nota:** O endpoint correto é `receberManifestoLote` (português correto) e não `receiverManifestoLote`.

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
[{
  "manNumero": "999999999999",
  "dataRecebimento": 1614080104836,
  "nomeMotorista": "motoris",
  "placaVeiculo": "ASD2234",
  "nomeResponsavelRecebimento": "nome conforme cadastro",
  "observacoes": "obs do recebimento",
  "listaManifestoResiduo": [
    {
      "resCodigoIbama": "010101",
      "marQuantidade": 5,
      "marQuantidadeRecebida": 10,
      "traCodigo": 4,
      "uniCodigo": 3,
      "marJustificativa": "justificativa se quantidade divergente",
      "resCodigoIbamaNovo": "020202",
      "traCodigoNovo": 17,
      "tieCodigo": 1,
      "tiaCodigo": 21,
      "claCodigo": 42
    }
  ]
}]
```

| Parâmetro | Tipo | Obrigatório | Descrição |
|-----------|------|-------------|-----------|
| `manNumero` | String | S | Número do MTR |
| `dataRecebimento` | Long | S | Timestamp ms do recebimento |
| `nomeMotorista` | String | S | Nome do motorista |
| `placaVeiculo` | String | S | Placa do veículo (sem hífen) |
| `nomeResponsavelRecebimento` | String | S | Nome do responsável (idêntico ao cadastrado) |
| `observacoes` | String | N | Observações |

**Campos de `listaManifestoResiduo[]`:**

| Parâmetro | Tipo | Obrigatório | Descrição |
|-----------|------|-------------|-----------|
| `resCodigoIbama` | String | S | Código IBAMA do resíduo |
| `marQuantidade` | Double | S | Quantidade original |
| `marQuantidadeRecebida` | Double | S | Quantidade recebida |
| `traCodigo` | Integer | S | Código do tratamento |
| `uniCodigo` | Integer | S | Código da unidade |
| `marJustificativa` | String | N | Obrigatório se quantidade divergente |
| `resCodigoIbamaNovo` | String | N | Substituição de resíduo |
| `traCodigoNovo` | Integer | N | Novo tratamento |
| `tieCodigo` | Integer | S | Estado físico |
| `tiaCodigo` | Integer | S | Acondicionamento |
| `claCodigo` | Integer | S | Classe |

---

## 15. Gerar CDF

### `POST /emiteCDF`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Modelo 1 — por lista de MTRs:**

```json
{
  "cerDataInicial": "20260701",
  "cerDataFinal": "20260729",
  "cerObservacao": "observacao",
  "parceiroDestinador": "99999",
  "cnpjDestinador": "99999999999999",
  "tipoCertificadoDestinacao": 1,
  "listaManifesto": [{"manNumero": "1111111111111"}],
  "listaParceiroGerador": [],
  "nomeResponsavel": "nome teste"
}
```

**Modelo 2 — por lista de geradores:**

```json
{
  "cerDataInicial": "20260701",
  "cerDataFinal": "20260729",
  "parceiroDestinador": "99999",
  "cnpjDestinador": "99999999999999",
  "tipoCertificadoDestinacao": 1,
  "listaManifesto": [],
  "listaParceiroGerador": [{"unidade": "88888", "cpfCnpj": "88888888888888"}],
  "nomeResponsavel": "nome teste"
}
```

> **Atenção:** O campo `cerDataInicial` possui "I" maiúsculo, não "l" minúsculo.

| Parâmetro | Tipo | Obrigatório | Descrição |
|-----------|------|-------------|-----------|
| `cerDataInicial` | String | S | Data inicial (yyyyMMdd ou timestamp ms) |
| `cerDataFinal` | String | S | Data final |
| `parceiroDestinador` | String | S | Código do destinador |
| `cnpjDestinador` | String | S | CNPJ do destinador |
| `tipoCertificadoDestinacao` | Integer | S | Fixo = 1 |
| `nomeResponsavel` | String | S | Nome do responsável (idêntico ao cadastrado) |
| `listaManifesto` | Array | N | Lista de MTRs |
| `listaParceiroGerador` | Array | N | Lista de geradores |

> Deve-se informar ou `listaManifesto` ou `listaParceiroGerador`.

---

## 16. Download do CDF

### `POST /downloadCertificado/{numeroCDF}`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request body:** `{}`

**Response:** Arquivo PDF do CDF (se HTTP 200 e não for mensagem de erro).

**Possíveis respostas de erro:**
1. Certificado não encontrado
2. WS Não disponível no momento.
3. O Token não pertence ao Gerador/Destinador.

---

## 17. Alteração de Recebimento

### `POST /alteraRecebimento`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
{
  "parCodigoDestinador": 99999,
  "remObservacao": "obs alteracao receb",
  "manifesto": {
    "manNumero": "9999999999999",
    "listaManifestoResiduo": [{
      "marQuantidade": 2.5,
      "marQuantidadeRecebida": 5.5,
      "resCodigoIbama": "010101",
      "claCodigo": 43,
      "claCodigoNovo": 42,
      "traCodigo": 17,
      "traCodigoNovo": 23
    }]
  }
}
```

---

## 18. Aceite de Alteração

### `POST /aceiteAlteracaoRecebimento`

> **Nota:** O endpoint correto é `aceiteAlteracaoRecebimento` (com "ca" em "Alteracao"), não `aceiteAlteraRecebimento`.

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
{
  "parCodigoGerador": 99999,
  "manNumero": "99999999999",
  "aceite": "a"
}
```

| Parâmetro | Tipo | Obrigatório | Descrição |
|-----------|------|-------------|-----------|
| `parCodigoGerador` | Integer | S | Código do gerador |
| `manNumero` | String | S | Número do manifesto |
| `aceite` | String | S | `"a"` (Aceita), `"r"` (Recusa) |

**Códigos de Retorno:**

| Código | Mensagem |
|--------|----------|
| 0 | Alteração aceita/recusada |
| 1 | Opção inválida |
| 2 | Manifesto não encontrado |
| 3 | Sem solicitação de alteração |
| 4 | Solicitação já finalizada |

---

## 19. Cancelar Manifesto em Lote

### `POST /cancelarManifestoLote`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
[
  { "manNumero": "99999999999", "justificativa": "teste de api" },
  { "manNumero": "99999999999", "justificativa": "teste de api" }
]
```

**Response:**

```json
{
  "mensagem": "Possui Erro em algum MTR",
  "objetoResposta": [
    {
      "manNumero": "240000878723",
      "justificativa": "jhdgajhg",
      "erro": true,
      "mensgaemRetorno": "Manifesto com situação diferente de Ativo. Não pode ser cancelado!",
      "situacao": null
    }
  ],
  "erro": true
}
```

**Possíveis mensagens de retorno:** Manifesto cancelado com sucesso / Apenas o Gerador pode cancelar / Manifesto não encontrado / Situação inválida para cancelamento.

---

## 20. Consulta MTR por Código de Referência

### `POST /retornaManifestoSeuCodigo`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
{
  "suaUnidade": 189788,
  "seuCodigo": "abc123"
}
```

| Parâmetro | Tipo | Obrigatório | Descrição |
|-----------|------|-------------|-----------|
| `suaUnidade` | Integer | S | Código da unidade do gerador |
| `seuCodigo` | String | S | Código de referência informado na criação |

**Response:** Mesmo formato da API 13 (Retorna Manifesto).

---

## 21. Gerar Manifesto Complementar

### `POST /salvarManifestoComplementar`

**Headers:**
```
Content-Type: application/json
Authorization: Bearer {token}
```

**Request:**

```json
{
  "nomeMotorista": "nome do motorista",
  "placaVeiculo": "asd-1212",
  "observacoes": "",
  "transportador": {
    "unidade": 189788,
    "cpfCnpj": "64229681000147"
  },
  "listaManifesto": [
    { "manNumero": "240002088896" }
  ]
}
```

| Parâmetro | Tipo | Obrigatório | Descrição |
|-----------|------|-------------|-----------|
| `nomeMotorista` | String | S | Nome do motorista |
| `placaVeiculo` | String | S | Placa do veículo |
| `observacoes` | String | N | Observações |
| `transportador.unidade` | Integer | S | Código da unidade do transportador |
| `transportador.cpfCnpj` | String | S | CPF/CNPJ do transportador |
| `listaManifesto` | Array | S | Lista de manifestos a complementar |

---

## Estrutura de Resposta Padrão

Todas as listagens (APIs 2-9) seguem o formato:

```json
{
  "mensagem": null,
  "objetoResposta": [ ... ],
  "erro": false
}
```

Endpoints de ação (APIs 10, 12, 14, 15, 17, 18, 19, 20, 21) seguem o formato com `restResponseValido`/`restResponseMensagem` nos objetos internos.

Endpoints de download (APIs 11, 16) retornam arquivos binários (PDF).

---

## Tabela de Métodos HTTP

| Operação | Método Real | Documentação Original |
|----------|-------------|----------------------|
| Todas as listagens (2-9) | **GET** | POST (incorreto) |
| gettoken (1) | POST | POST (correto) |
| salvarManifestoLote (10) | POST | POST (correto) |
| downloadManifesto (11) | POST | POST (correto) |
| cancelarManifesto (12) | POST | POST (correto) |
| retornaManifesto (13) | GET | GET (correto) |
| receberManifestoLote (14) | POST | POST (correto) |
| emiteCDF (15) | POST | POST (correto) |
| downloadCertificado (16) | POST | POST (correto) |
| alteraRecebimento (17) | POST | POST (correto) |
| aceiteAlteracaoRecebimento (18) | POST | POST (correto) |
| cancelarManifestoLote (19) | POST | POST (correto) |
| retornaManifestoSeuCodigo (20) | POST | POST (correto) |
| salvarManifestoComplementar (21) | POST | POST (correto) |

---

## Erratas vs Documentação Original

| Item | Documentação Original | Real |
|------|----------------------|------|
| Método listagens | POST | **GET** |
| `resCodigolbama` | lower L | `resCodigoIbama` (upper I) |
| `cerDatalnicial` | lower L | `cerDataInicial` (upper I) |
| `receiverManifestoLote` | R plus ER | **`receberManifestoLote`** |
| `aceiteAlteraRecebimento` | missing `ca` | **`aceiteAlteracaoRecebimento`** |
| `EstadoFísico` (acento) | with accent | **`EstadoFisico`** (no accent) |
| `objetoResposta` Bearer | corrupted JWT | valid JWT |
