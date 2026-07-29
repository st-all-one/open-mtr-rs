# Services

Cada domínio tem um service em `core/src/services/`. Services são `#[derive(Clone)]` e `Send + Sync`.

## ManifestoService

```rust
pub async fn listar(di, df, tipo, page, size)
    -> Result<(Vec<ManifestoResumo>, Option<i64>), MtrError>

pub async fn listar_todos(di, df, tipo)
    -> Result<Vec<ManifestoResumo>, MtrError>

pub async fn cancelar(man_numero, justificativa)
    -> Result<CancelamentoResponse, MtrError>

pub async fn download_pdf(man_numero)
    -> Result<Vec<u8>, MtrError>

pub async fn salvar_pdf(man_numero, caminho)
    -> Result<(), MtrError>
```

### ManifestoListBuilder (typestate)

```rust
ManifestoListBuilder<StFaltando>
    .periodo(inicio, fim)   // valida fim >= inicio → StPronto
    .tipo_parceiro(t)
    .pagina(p)
    .tamanho_pagina(s)
    .build()                // só disponível em StPronto
    → ApiwsListaManifestoDTO
```

## SalvarManifestoLoteBuilder (typestate)

Em `services/salvar_manifesto.rs`:

```rust
SalvarManifestoLoteBuilder<SemResiduos>
    .identificacao(man_numero, data_emissao, ...)
    .transportador(cnpj, razao_social, ...)
    .add_residuo(...)      // transiciona para ComResiduos
    .build()               // só em ComResiduos
```

## RecebimentoService

```rust
pub async fn registrar_lote(itens)
    -> Result<Vec<RecebimentoLoteItem>, MtrError>

pub async fn solicitar_alteracao(payload)
    -> Result<serde_json::Value, MtrError>

pub async fn aceitar_alteracao(payload)
    -> Result<AceiteAlteracaoResponse, MtrError>
```

## CdfService

```rust
pub async fn emitir(payload)
    -> Result<CDFEmissaoObjeto, MtrError>

pub async fn emitir_lr(payload)
    -> Result<CDFEmissaoObjeto, MtrError>

pub async fn download(cdf_codigo, formato)
    -> Result<Vec<u8>, MtrError>

pub async fn salvar(cdf_codigo, formato, caminho)
    -> Result<(), MtrError>
```

## LogisticaReversaService

```rust
pub async fn listar_origem(di, df)
    -> Result<Value, MtrError>

pub async fn listar_processo(di, df)
    -> Result<Value, MtrError>

pub async fn obter_origem_completo(man_numero)
    -> Result<MtrOrigemCompletoObjeto, MtrError>

pub async fn obter_processo(man_numero)
    -> Result<Value, MtrError>

pub async fn alterar_titularidade(payload)
    -> Result<Value, MtrError>
```

## IntegracaoEstadualService

```rust
pub async fn integrar_manifestos(manifestos)
    -> Result<RespostaIntegracaoObjeto, MtrError>

pub async fn integrar_recebimento(recebimento)
    -> Result<Value, MtrError>
```

## TabelasService

```rust
pub async fn listar_todos()
    -> Result<Vec<TipoAcondicionamento>, MtrError>

pub async fn listar_por_estado_fisico(tie_codigo)
    -> Result<Vec<TipoAcondicionamento>, MtrError>

pub fn get_compativeis(tie_codigo) -> &'static [i32]

pub fn get_nome_estado_fisico(tie_codigo) -> &'static str
```

Cache interno de 24h via `Arc<RwLock<Option<CacheEntry>>>`.

## AuthService (deprecated)

```rust
#[deprecated]
pub async fn autenticar(creds) -> Result<TokenObjeto, MtrError>
```

Use `MtrClient` direto com `integration_token`.
