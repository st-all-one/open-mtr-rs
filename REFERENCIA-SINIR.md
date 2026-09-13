# Referência da API SINIR MTR — mtr-sinir-rs

## 1. Arquitetura Geral

```
                    ┌─────────────────────────────────────┐
                    │        Aplicação Consumidora         │
                    │    (ERP, sistema de gestão de       │
                    │     resíduos, integrador estadual)   │
                    └────────────────┬────────────────────┘
                                     │
                    ┌────────────────▼────────────────────┐
                    │          mtr-sinir-rs (Rust)         │
                    │                                     │
                    │  ┌─────────────────────────────┐    │
                    │  │       Prelude Público        │    │
                    │  │  (9 Services, 11 Schema mods)│    │
                    │  └─────────────┬───────────────┘    │
                    │                │                     │
                    │  ┌─────────────▼───────────────┐    │
                    │  │       MtrClient (core)       │    │
                    │  │  • TokenManager (JWT auto)   │    │
                    │  │  • Retry (3x exp backoff)    │    │
                    │  │  • Cache (Tabelas, 24h TTL)  │    │
                    │  └─────────────┬───────────────┘    │
                    └────────────────┬────────────────────┘
                                     │
                    ┌────────────────▼────────────────────┐
                    │    SINIR API — admin.sinir.gov.br    │
                    │         HTTPS :443                   │
                    │    ┌────────────────────────┐        │
                    │    │   /api/* (26 endpoints) │        │
                    │    └────────────────────────┘        │
                    └─────────────────────────────────────┘
```

## 2. Autenticação — Fluxo JWT

```
 Integração ──► POST /api/token ──► JWT Bearer ──► Header automático em toda request
     │                              (exp 8-24h)         │
     └─── integration_token ───────┘                     │
                                                         ▼
                                               Se 401 → refresh automático
                                                         │
                                               └── retry 1x com novo token
```

| Etapa | Mecanismo | Detalhes |
|-------|-----------|----------|
| 1. Config | `MtrConfig::builder().integration_token("...")` | Token fixo do SINIR |
| 2. Exchange | `POST /api/token` com `Authorization: Bearer {token}` | Devolve JWT + `exp` |
| 3. Cache | `TokenManager` guarda: `access_token`, `expires_at` | Margem de segurança: 60s |
| 4. Refresh | `TokenManager::refresh()` se expirado ou 401 | Fallback 8h se JWT sem `exp` |
| 5. Header | `Authorization: Bearer {jwt}` em toda request | Transparente ao usuário |

## 3. Estrutura da Resposta Padrão

```json
{
    "erro": false,
    "objetoResposta": { ... },
    "mensagem": null,
    "totalRecords": 150
}
```

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `erro` | `bool` | `false` = sucesso, `true` = erro de negócio |
| `objetoResposta` | `T \| null` | Payload principal (nullable mesmo em sucesso) |
| `mensagem` | `string \| null` | Mensagem de erro ou aviso |
| `totalRecords` | `int \| null` | Total de registros (lista/paginação) |

Mapeamento Rust: `ObjetoRespostaRest<T>` — genérico, presente em todo endpoint.

## 4. Mapa Completo de Endpoints (26)

### 4.1 Infraestrutura

| # | Método | Path | Service (Rust) | Função |
|---|--------|------|----------------|--------|
| 1 | `POST` | `/token` | `TokenManager` (interno) | Troca `integration_token` por JWT Bearer |

### 4.2 Manifesto (MTR)

| # | Método | Path | Service | Função |
|---|--------|------|---------|--------|
| 2 | `POST` | `/listaManifesto` | `ManifestoService::listar` | Lista MTRs por período + perfil |
| 3 | `POST` | `/cancelarManifesto` | `ManifestoService::cancelar` | Cancela MTR com justificativa |
| 4 | `POST` | `/downloadManifesto/{num}` | `ManifestoService::download_pdf` | Download do PDF do MTR |

### 4.3 Recebimento

| # | Método | Path | Service | Função |
|---|--------|------|---------|--------|
| 5 | `POST` | `/receberManifestoLote` | `RecebimentoService::registrar_lote` | Registra recebimento em lote |
| 6 | `POST` | `/alteraRecebimento` | `RecebimentoService::solicitar_alteracao` | Solicita alteração de recebimento |
| 7 | `POST` | `/aceiteAlteracaoRecebimento` | `RecebimentoService::aceitar_alteracao` | Aceita/recusa alteração |

### 4.4 Certificado de Destinação Final (CDF)

| # | Método | Path | Service | Função |
|---|--------|------|---------|--------|
| 8 | `POST` | `/emiteCDF` | `CdfService::emitir` | Emite CDF para MTRs |
| 9 | `POST` | `/emiteCDFLR` | `CdfService::emitir_lr` | Emite CDF para logística reversa |
| 10 | `POST` | `/downloadCertificado/{cod}` | `CdfService::download` | Download PDF/XML do CDF |

### 4.5 Logística Reversa

| # | Método | Path | Service | Função |
|---|--------|------|---------|--------|
| 11 | `POST` | `/listaMtrOrigem` | `LogisticaReversaService::listar_origem` | Lista MTRs de origem LR |
| 12 | `POST` | `/mtrOrigemCompleto` | `LogisticaReversaService::obter_origem_completo` | Detalhes + rastreio MTR origem |
| 13 | `POST` | `/alterarManifestoLROrigem` | `LogisticaReversaService::alterar_titularidade` | Altera titularidade LR |
| 14 | `POST` | `/listaMtrProcesso` | `LogisticaReversaService::listar_processo` | Lista MTRs de processo LR |
| 15 | `POST` | `/mtrProcesso` | `LogisticaReversaService::obter_processo` | Detalhes MTR processo |

### 4.6 Integração Estadual

| # | Método | Path | Service | Função |
|---|--------|------|---------|--------|
| 16 | `POST` | `/integraMTREstadual` | `IntegracaoEstadualService::integrar_manifestos` | Envia MTRs para estado |
| 17 | `POST` | `/integraRecebimentoMTREstadual` | `IntegracaoEstadualService::integrar_recebimento` | Envia recebimento para estado |

### 4.7 Tabelas de Apoio (GET — cacheáveis)

| # | Método | Path | Service | Cache |
|---|--------|------|---------|-------|
| 18 | `GET` | `/retornaListaAcondicionamento` | `TabelasService::listar_todos` | 24h (15 tipos) |
| 19 | `GET` | `/retornaListaAcondicionamentoPorEstadoFisico/{cod}` | `TabelasService::listar_por_estado_fisico` | 24h por `tie_codigo` |
| 20 | `GET` | `/retornaListaUnidade` | `TabelasService::listar_unidades` | Sem cache (5) |
| 21 | `GET` | `/retornaListaEstadoFisico` | `TabelasService::listar_estados_fisicos` | Sem cache (4) |
| 22 | `GET` | `/retornaListaClasse` | `TabelasService::listar_classes` | Sem cache (18) |
| 23 | `GET` | `/retornaListaTratamento` | `TabelasService::listar_tratamentos` | Sem cache (35) |
| 24 | `GET` | `/retornaListaResiduo` | `ResiduoService::listar_todos` | Sem cache (895) |

### 4.8 Emissão de MTR (POST)

| # | Método | Path | Service | Função |
|---|--------|------|---------|--------|
| 25 | `POST` | `/salvarManifestoLote` | `ManifestoLoteService::salvar_lote` | Emite 1..N MTRs |

## 5. Interdependências Schema → Endpoint

### 5.1 Composição de Schemas

```
SalvarManifestoLoteRequest
├── gerador: ParteManifesto
│   ├── cpf_cnpj: String
│   └── unidade: i32
├── transportador: ParteManifesto
├── destinador: ParteManifesto
├── armazenador_temporario: Option<ParteManifesto>
├── lista_residuos: Vec<ResiduoManifestoRequest>
│   ├── res_codigo_ibama: String  ← ResiduoIbama.codigo_ibama
│   ├── uni_codigo: i32           ← UnidadeMedida.codigo
│   ├── tie_codigo: i32           ← EstadoFisico.codigo
│   ├── cla_codigo: i32           ← ClasseResiduo.codigo
│   ├── tia_codigo: i32           ← TipoAcondicionamento.codigo
│   ├── tra_codigo: i32           ← TratamentoResiduo.codigo
│   └── quantidade: f64
├── possui_armazenamento_temporario: bool
├── nome_responsavel: String
└── observacoes: Option<String>
```

### 5.2 Matriz Schema × Endpoint

| Schema | Endpoints que usam | Campos chave |
|--------|-------------------|--------------|
| `SalvarManifestoLoteRequest` | #25 | `lista_residuos`, partes |
| `ResiduoManifestoRequest` | #25 | `res_codigo_ibama`, `uni_codigo`, `tie_codigo`, `cla_codigo`, `tia_codigo`, `tra_codigo` |
| `ParteManifesto` | #25 | `cpf_cnpj`, `unidade` |
| `ApiwsListaManifestoDTO` | #2 | `data_inicial`, `data_final`, `tipo_parceiro` |
| `ManifestoResumo` | #2 | `man_numero`, `man_status` |
| `CancelaManifestoDTO` | #3 | `man_numero`, `justificativa` |
| `ApiwsManifestoDeRecebimentoDTO` | #5 | `man_numero`, `lista_residuos_recebidos` |
| `ResiduoRecebido` | #5 | `res_codigo_ibama`, `quantidade_recebida` |
| `ApiwsAlteraRecebimentoDTO` | #6 | `man_numero`, `justificativa`, `lista_residuos_alterados` |
| `AceiteAlteracaoDTO` | #7 | `man_numero`, `aceite` |
| `ApiwsCertificadoDestinacaoDTO` | #8, #9 | `lista_manifestos`, `data_emissao` |
| `ApiwsManifestoDTO` | #16 | `man_numero`, `lista_residuos`, partes completas |
| `ApiwsAlteraManifestoOrigemDTO` | #13 | `man_numero`, `novo_titular` |
| `MtrLrDataDTO` | #11, #14 | `data_inicial`, `data_final` |
| `MtrOrigemCompletoDTO` | #12 | `man_numero` |
| `MtrOrigemCompletoObjeto` | #12 → response | `mtr`, `rastreio: Vec<RastreioEvento>` |
| `ObjetoRespostaRest<T>` | Todos | `erro`, `objeto_resposta`, `mensagem` |
| `ResiduoIbama` | #24 | `codigo_ibama`, `descricao` |
| `TipoAcondicionamento` | #18, #19 | `codigo`, `descricao` |
| `UnidadeMedida` | #20 | `codigo`, `descricao`, `sigla` |
| `EstadoFisico` | #21 | `codigo`, `descricao` |
| `ClasseResiduo` | #22 | `codigo`, `descricao`, `resolucao` |
| `TratamentoResiduo` | #23 | `codigo`, `descricao` |

## 6. Camada de Serviços — Padrão de Implementação

```
┌──────────────────────────────────────────────────┐
│                  Service Layer                    │
│                                                   │
│  Cada Service:                                    │
│  • Construtor: fn new(client: MtrClient) -> Self  │
│  • Métodos async: Result<T, MtrError>             │
│  • Request → ObjetoRespostaRest<T> → unwrap → T   │
│  • Erro mapeado com ErrorCategory + ErrorContext   │
│                                                    │
│  Exceções:                                          │
│  • TabelasService: adiciona cache 24h              │
│  • ManifestoService: paginação iterator            │
│  • RecebimentoService: builder pattern             │
└──────────────────────────────────────────────────┘
```

### 6.1 Mapa Service → Endpoints

| Service | Endpoints | Padrão |
|---------|-----------|--------|
| `ManifestoService` | #2, #3, #4 | CRUD + download |
| `RecebimentoService` | #5, #6, #7 | Lote + alteração |
| `RecebimentoBuilder` | #5 (auxiliar) | Builder fluent |
| `CdfService` | #8, #9, #10 | Emissão + download (Pdf/Xml) |
| `LogisticaReversaService` | #11, #12, #13, #14, #15 | Consulta + alteração |
| `IntegracaoEstadualService` | #16, #17 | Envio externo |
| `TabelasService` | #18, #19, #20, #21, #22, #23 | GET + cache |
| `ResiduoService` | #24 | GET + filtro perigosos |
| `ManifestoLoteService` | #25 | Emissão lote |

### 6.2 Serviços Auxiliares (não expostos no prelude)

| Função | Localização | Propósito |
|--------|-------------|-----------|
| `validar_acondicionamento()` | `tabelas.rs` | Verifica compatibilidade acondicionamento × estado físico |
| `integrar_lote_inteligente()` | `integracao_estadual.rs` | Envio com particionamento automático |
| `with_retry()` | `retry.rs` | Retry genérico 3x exp backoff |
| `validar_emissao()` | `cdf.rs` | Valida payload antes de emitir CDF |

## 7. Máquina de Estados do MTR

```
                     ┌──────────────┐
                     │   Provisório  │
                     │  (rascunho)   │
                     └──────┬───────┘
                            │
                            ▼
                     ┌──────────────┐
                  ──►│   Emitido     │◄── #25 salvarManifestoLote
                 │   └──────┬───────┘
                 │          │
                 │          ▼
                 │   ┌──────────────┐
                 │   │  Recebido     │── #5 receberManifestoLote
                 │   └──────┬───────┘
                 │          │
                 │          ▼
                 │   ┌──────────────┐       ┌──────────────┐
                 │   │   CDF Emitido │──#8──►  CDF Download │ #10
                 │   └──────┬───────┘       └──────────────┘
                 │          │
                 │          ▼
                 │   ┌──────────────┐
                 │   │  Integrado    │── #16 integraMTREstadual
                 │   │  (Estadual)   │
                 │   └──────────────┘
                 │
                 │   ┌──────────────┐
                 └───│  Cancelado    │── #3 cancelarManifesto
                     └──────────────┘
```

### Transições possíveis

| De | Para | Endpoint | Condição |
|----|------|----------|----------|
| Provisório | Emitido | `#25 POST /salvarManifestoLote` | Dados válidos, sem MTR provisório pendente |
| Emitido | Recebido | `#5 POST /receberManifestoLote` | Destinador confirma recebimento |
| Emitido | Cancelado | `#3 POST /cancelarManifesto` | Justificativa obrigatória |
| Recebido | CDF Emitido | `#8 POST /emiteCDF` | Todos resíduos recebidos |
| Recebido | Integrado | `#16 POST /integraMTREstadual` | Estado exige integração |
| Recebido | Alteração | `#6 POST /alteraRecebimento` → `#7 POST /aceiteAlteracaoRecebimento` | Duas etapas |

## 8. Tabelas de Apoio — Valores Estáticos vs API

| Tabela | API (endpoint) | Fallback offline | Quantidade |
|--------|----------------|------------------|------------|
| Acondicionamento | `#18 GET /retornaListaAcondicionamento` | `COMPATIBILIDADE_ACOND` (HashMap de compatibilidades) | 15 tipos |
| Estado Físico | `#21 GET /retornaListaEstadoFisico` | `ESTADOS_FISICOS` (HashMap codigo → nome) | 4 estados (API) / 5 (fallback) |
| Unidade | `#20 GET /retornaListaUnidade` | N/A | 5 |
| Classe | `#22 GET /retornaListaClasse` | N/A | 18 |
| Tratamento | `#23 GET /retornaListaTratamento` | N/A | 35 |
| Resíduo IBAMA | `#24 GET /retornaListaResiduo` | N/A | 895 (448 perigosos) |

> ⚠️ Os fallbacks offline (`COMPATIBILIDADE_ACOND`, `ESTADOS_FISICOS`) são aproximações estáticas.
> Os valores da API real **prevalecem** sobre os fallbacks.

## 9. Mapa de Erros HTTP → Categoria

| HTTP Status | ErrorCategory | Disparado por |
|-------------|---------------|---------------|
| 400 | `ValidationError` | Payload inválido, campos obrigatórios faltando |
| 401 | `AuthenticationFailed` | Token JWT expirado ou inválido |
| 403 | `AuthorizationDenied` | Perfil sem acesso ao recurso |
| 404 | `ResourceNotFound` | MTR, CDF ou certificado inexistente |
| 409 | `Conflict` | MTR já cancelado, CDF já emitido |
| 422 | `ValidationError` | Erro de negócio (ex.: resíduo incompatível) |
| 429 | `RateLimited` | Muitas requests em curto período |
| 500+ | `InternalError` | Erro interno do SINIR |
| 503/504 | `Timeout` / `NetworkError` | Timeout ou falha de rede |

### Comportamento de Retry

```
with_retry(f, max_retries=3)
  ├── 429 Too Many Requests → retry (1s, 2s, 4s)
  ├── 5xx Server Error      → retry (1s, 2s, 4s)
  ├── timeout/disconnect    → retry (1s, 2s, 4s)
  ├── 401 Unauthorized      → refresh token + retry 1x
  └── 4xx (exceto 401,429)  → erro imediato (sem retry)
```

## 10. Pirâmide da Integração SINIR

```
                    ┌──────────────────────────────────┐
                    │   Aplicações de Negócio           │
                    │  (ERP, sistemas de gestão         │
                    │   ambiental, portais estaduais)   │
                    └──────────────┬───────────────────┘
                                   │
                    ┌──────────────▼───────────────────┐
                    │       mtr-sinir-rs (Rust)        │
                    │  • 9 Services, 11 Schema mods    │
                    │  • Builder pattern (Recebimento) │
                    │  • Cache layer (Tabelas, 24h)    │
                    │  • Retry 3x exp backoff          │
                    │  • Download Pdf/Xml              │
                    └──────────────┬───────────────────┘
                                   │
                    ┌──────────────▼───────────────────┐
                    │   MtrClient + TokenManager        │
                    │  • Auto-auth JWT                  │
                    │  • Refresh transparente           │
                    │  • ObjetoRespostaRest<T>          │
                    │  • Error mapping categórico       │
                    └──────────────┬───────────────────┘
                                   │
                    ┌──────────────▼───────────────────┐
                    │   SINIR API — HTTPS :443          │
                    │  • 26 endpoints                  │
                    │  • JWT Bearer (8-24h)             │
                    │  • JSON unificado                 │
                    │  • CORS restricted                │
                    └──────────────┬───────────────────┘
                                   │
                    ┌──────────────▼───────────────────┐
                    │   Transport Layer                  │
                    │  • TLS 1.2+                       │
                    │  • HTTP/1.1 ou HTTP/2              │
                    │  • TCP :443                       │
                    └──────────────────────────────────┘
```

## 11. Exemplos de Uso

### 11.1 Fluxo Completo — Emitir MTR

```rust
use mtr_sinir_core::prelude::*;

async fn exemplo_emissao() -> Result<(), MtrError> {
    let cfg = MtrConfig::builder()
        .base_url("https://admin.sinir.gov.br/api")
        .integration_token(&std::env::var("MTR_TOKEN")?)
        .build()?;
    let client = MtrClient::new(cfg)?;
    let mtr = ManifestoLoteService::new(client);

    let req = vec![SalvarManifestoLoteRequest {
        possui_armazenamento_temporario: false,
        nome_responsavel: "João Silva".into(),
        gerador: ParteManifesto { cpf_cnpj: "00.000.000/0001-00".into(), unidade: 130204 },
        transportador: ParteManifesto { cpf_cnpj: "00.000.000/0001-00".into(), unidade: 130147 },
        destinador: ParteManifesto { cpf_cnpj: "00.000.000/0001-00".into(), unidade: 130204 },
        armazenador_temporario: None,
        nome_motorista: None,
        placa_veiculo: None,
        data_expedicao: None,
        observacoes: Some("Coleta rotineira".into()),
        tipo_manifesto: Some(0),
        sim_codigo: None,
        lista_residuos: vec![ResiduoManifestoRequest {
            res_codigo_ibama: "150203".into(),
            quantidade: 500.0,
            uni_codigo: 1,
            tie_codigo: 4,
            cla_codigo: 43,
            tia_codigo: 2,
            tra_codigo: 17,
            descricao_interna: None,
            codigo_interno: None,
        }],
    }];

    let resp = mtr.salvar_lote(&req).await?;
    println!("OK: erro_nacional={}", resp.erro_nacional);
    Ok(())
}
```

### 11.2 Receber + Aceitar Alteração

```rust
async fn exemplo_recebimento(client: MtrClient) -> Result<(), MtrError> {
    let rec = RecebimentoService::new(client);

    let lote = vec![
        RecebimentoBuilder::new()
            .manifesto("SP-123456")
            .data(TimestampMs(1711400000000))
            .responsavel("Maria Souza")
            .add_residuo("150203", 500.0, 1)
            .build()?
    ];
    let resultado = rec.registrar_lote(&lote).await?;

    let alteracao = ApiwsAlteraRecebimentoDTO {
        man_numero: "SP-123456".into(),
        justificativa: "Recontagem".into(),
        lista_residuos_alterados: vec![ResiduoAlterado {
            res_codigo_ibama: "150203".into(),
            nova_quantidade: 480.0,
            uni_codigo: 1,
            observacoes: Some("Diferença de 20kg".into()),
        }],
    };
    rec.solicitar_alteracao(&alteracao).await?;

    rec.aceitar_alteracao(&AceiteAlteracaoDTO {
        man_numero: "SP-123456".into(),
        aceite: true,
        observacoes: None,
    }).await?;
    Ok(())
}
```

### 11.3 Tabelas com Validação

```rust
async fn exemplo_tabelas(client: MtrClient) -> Result<(), MtrError> {
    let tab = TabelasService::new(client);

    // API real (preferencial)
    let unidades = tab.listar_unidades().await?;       // 5 unidades
    let estados = tab.listar_estados_fisicos().await?;  // 4 estados
    let classes = tab.listar_classes().await?;          // 18 classes

    // Fallback offline
    let nome = TabelasService::get_nome_estado_fisico(4); // "Sólido" (ou "Desconhecido")

    // Validação combinada
    let compativeis = TabelasService::get_compativeis(4); // acond. compatíveis
    Ok(())
}
```

## 12. Considerações de Segurança

| Aspecto | Prática | Camada |
|---------|---------|--------|
| Transporte | TLS 1.2+ obrigatório | Transport |
| Autenticação | JWT Bearer, renovação automática | Auth |
| Token storage | `integration_token` em `MtrConfig`, nunca logado | Config |
| Rate limiting | Retry com backoff exponencial (1s/2s/4s) | Client |
| Validação | `validar_cnpj()` antes de enviar | Schema |
| Cache | Dados de tabela apenas (sem PII) | Service |
| Timeout | 30s default, configurável | Client |
| Error handling | Erros categorizados, sem vazar detalhes internos | Core |

---

## 13. Recebimento em Lote — contrato validado

> Validado ao vivo contra `admin.sinir.gov.br` (conta destinadora UBA, unid. 130204).
> O `SinirAdapter::receber_mtrs` serializa exatamente este formato.

### `POST /receberManifestoLote`

O corpo é **um array JSON** — mesmo para um único MTR. Enviar um objeto solto
resulta em HTTP 400 (antes do Jackson). Esta era a causa de o recebimento SINIR
nunca funcionar no adapter.

```json
[{
  "manNumero": "251032769556",
  "dataRecebimento": 1789330800000,
  "nomeMotorista": "WELLINGTON",
  "placaVeiculo": "RLZ4A36",
  "nomeResponsavelRecebimento": "Norlu Apoio",
  "observacoes": "recebimento via open-mtr-rs",
  "listaManifestoResiduos": [
    {
      "resCodigoIbama": "Classe A",
      "marQuantidade": 5,
      "marQuantidadeRecebida": 5,
      "uniCodigo": 3,
      "tieCodigo": 4,
      "claCodigo": 11,
      "tiaCodigo": 4,
      "traCodigo": 43,
      "marJustificativa": "Substituição do resíduo Classe A por 170107 (RCC)",
      "resCodigoIbamaNovo": "170107",
      "traCodigoNovo": 43
    }
  ]
}]
```

| Campo | Tipo | Obrigatório | Observação |
|---|---|---|---|
| `manNumero` | String | S | Número do MTR |
| `dataRecebimento` | Long | S | Timestamp em ms (aceita data retroativa) |
| `nomeMotorista` | String | S | Ecoado pela API |
| `placaVeiculo` | String | S | Sem hífen (`RLZ4A36`) |
| `nomeResponsavelRecebimento` | String | S | **Idêntico ao responsável cadastrado** |
| `observacoes` | String | N | |
| `listaManifestoResiduos` | Array | S | **Plural no SINIR** (o SIGOR usa singular `listaManifestoResiduo`) |

Campos de `listaManifestoResiduos[]`:

| Campo | Tipo | Obrigatório | Observação |
|---|---|---|---|
| `resCodigoIbama` | String | S | Código **declarado no manifesto** (ex.: `"Classe A"`) |
| `marQuantidade` | Double | S | Quantidade original |
| `marQuantidadeRecebida` | Double | S | Quantidade recebida |
| `uniCodigo` / `tieCodigo` / `claCodigo` / `tiaCodigo` / `traCodigo` | Integer | S | Unidade / estado físico / classe / acondicionamento / tratamento |
| `marJustificativa` | String | Cond. | Obrigatória em divergência de quantidade **ou** substituição |
| `resCodigoIbamaNovo` | String | N | Substituição do resíduo |
| `traCodigoNovo` / `claCodigoNovo` | Integer | N | Novos parâmetros da substituição |
| `marNumeroONU` / `marClasseRisco` / `marNomeEmbarque` | String | N | Apenas resíduo perigoso |
| `greCodigo` | Integer | N | Grupo de embalagem (**inteiro**; string causa HTTP 400) |

### Resposta

```json
{
  "erro": false,
  "mensagem": "Manifesto recebido com sucesso!",
  "totalRecords": 0,
  "objetoResposta": [
    {
      "manNumero": "251032769556",
      "manCodigo": 32853390,
      "restResponseValido": true,
      "restResponseMensagem": "Manifesto recebido com sucesso!"
    }
  ]
}
```

O adapter correlaciona cada item de `objetoResposta` ao MTR de entrada por
`manNumero` e usa `restResponseValido`/`restResponseMensagem`. Quando a API
responde **HTTP 200 com `erro: true`**, o lote inteiro é marcado como erro de
negócio (com `mensagem`).

### Regras de negócio (descobertas ao vivo)

1. **Responsável cadastrado.** `nomeResponsavelRecebimento` precisa ser um
   responsável **cadastrado na unidade**; normalmente é o *"nome especial"* do
   usuário logado (não o login). Caso contrário:
   `"Responsável pelo recebimento não encontrado!"`.
2. **Resíduo casado com o manifesto.** `resCodigoIbama` deve ser o código
   declarado no manifesto (ex.: `"Classe A"`). Enviar outro código sem
   `resCodigoIbamaNovo` falha com
   `"Resíduo/Quantidade/Tratamento não encontrado"`.
3. **Substituição no próprio item.** Para trocar o resíduo, enviam
   `resCodigoIbamaNovo` (+ `traCodigoNovo`/`claCodigoNovo`) e
   `marJustificativa`. **Não** é necessário `/alteraRecebimento` nem duas
   entradas. O original não é removido: passa a constar com o novo código.
4. **Período de consulta ≤ 30 dias.** Períodos maiores recebem HTTP 400
   (`"O período de pesquisa não pode ser superior a 30 dias"`); o adapter
   valida antes e devolve erro claro (422).
5. **`listar_classes_por_residuo` não é suportado** no SINIR (HTTP 501);
   use `listar_classes` e associe localmente.

### Exemplo wiremock (corpo)

O teste `core/tests/wiremock_sinir.rs` valida o corpo enviado via
`server.received_requests()`: array, nomes corretos, ausência dos campos antigos
(`nomeResponsavel`, `listaResiduosRecebidos`, `quantidadeRecebida`) e presença de
`resCodigoIbamaNovo`/`marJustificativa`.

---

## A. Apêndice — Catálogo de Schemas

### A.1 Schemas de Requisição

| Struct | Submódulo | Serialização |
|--------|-----------|-------------|
| `ApiwsLoginDTO` | `auth` | `{"usuario": "...", "senha": "..."}` |
| `ApiwsListaManifestoDTO` | `manifesto` | `{"dataInicial": "...", "dataFinal": "...", "tipoParceiro": 5}` |
| `CancelaManifestoDTO` | `manifesto` | `{"manNumero": "...", "justificativa": "..."}` |
| `ApiwsManifestoDeRecebimentoDTO` | `recebimento` | `{"manNumero": "...", "dataRecebimento": 1711400000, ...}` |
| `ApiwsAlteraRecebimentoDTO` | `recebimento` | `{"manNumero": "...", "justificativa": "...", ...}` |
| `AceiteAlteracaoDTO` | `recebimento` | `{"manNumero": "...", "aceite": true, ...}` |
| `ApiwsCertificadoDestinacaoDTO` | `cdf` | `{"listaManifestos": [...], "dataEmissao": 1711400000, ...}` |
| `SalvarManifestoLoteRequest` | `salvar_manifesto` | `{"possuiArmazenamentoTemporario": false, ...}` |
| `ApiwsManifestoDTO` | `integracao_estadual` | `{"manNumero": "...", "gerador": {...}, ...}` |
| `ApiwsAlteraManifestoOrigemDTO` | `logistica_reversa` | `{"manNumero": "...", "novoTitular": "...", ...}` |
| `MtrLrDataDTO` | `logistica_reversa` | `{"dataInicial": "...", "dataFinal": "..."}` |

### A.2 Schemas de Resposta

| Struct | Submódulo | Descrição |
|--------|-----------|-----------|
| `TokenObjeto` | `auth` | `{"token": "...", "_tipo": "Bearer", "expiraEm": 1711400000}` |
| `ManifestoResumo` | `manifesto` | `{"manNumero": "SP-...", "manStatus": "EMITIDO", ...}` |
| `CancelamentoResponse` | `manifesto` | `{"manNumero": "...", "status": "cancelado", ...}` |
| `RecebimentoLoteItem` | `recebimento` | `{"manNumero": "...", "recebido": true, "mensagem": "OK"}` |
| `AceiteAlteracaoResponse` | `recebimento` | `{"manNumero": "...", "alteracaoAceita": true, ...}` |
| `CDFEmissaoObjeto` | `cdf` | `{"cdfCodigo": 1234, "cdfNumero": "...", ...}` |
| `MtrOrigemCompletoObjeto` | `logistica_reversa` | `{"mtr": {...}, "rastreio": [...]}` |
| `RespostaIntegracaoObjeto` | `integracao_estadual` | `{"manifestosProcessados": 5, ...}` |
| `SalvarManifestoResponse` | `salvar_manifesto` | `{"erroNacional": false, "respostas": [...]}` |
| `RespostaManifestoLote` | `salvar_manifesto` | `{"codigoGerado": 123, "valido": true, ...}` |
| `TipoAcondicionamento` | `tabelas` | `{"tiaCodigo": 1, "tiaDescricao": "..."}` |
| `UnidadeMedida` | `tabelas` | `{"uniCodigo": 1, "uniDescricao": "...", "uniSigla": "UN"}` |
| `EstadoFisico` | `tabelas` | `{"tieCodigo": 1, "tieDescricao": "SEMISSÓLIDO"}` |
| `ClasseResiduo` | `tabelas` | `{"claCodigo": 1, "claDescricao": "CLASSE I", ...}` |
| `TratamentoResiduo` | `tabelas` | `{"traCodigo": 1, "traDescricao": "..."}` |
| `ResiduoIbama` | `residuo` | `{"resCodigoIbama": "150203", "resDescricao": "..."}` |
| `ObjetoRespostaRest<T>` | `common` | `{"erro": false, "objetoResposta": T, "mensagem": null, ...}` |

---

*Documentação gerada a partir da implementação e validação real contra `https://admin.sinir.gov.br/api`. 26 endpoints, 11 módulos de schema, 9 services.*
