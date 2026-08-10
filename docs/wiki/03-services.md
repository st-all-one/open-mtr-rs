# Adaptadores (MtrAdapter)

Não existe mais camada de services. Cada sistema implementa o trait `MtrAdapter`.

## Trait MtrAdapter

```rust
#[async_trait]
pub(crate) trait MtrAdapter: Send + Sync {
    fn system(&self) -> System;
    fn nome(&self) -> &'static str;

    // Auth
    async fn autenticar_token(&self, ctx: &MtrCtx) -> Result<String, OpenMTRError>;
    async fn autenticar_credenciais(&self, ctx: &MtrCtx, params: &AuthParams) -> Result<String, OpenMTRError>;

    // 15 métodos de negócio (todos retornam LoteResult<T>)
    async fn consultar_mtrs(&self, ctx: &MtrCtx, filtro: &FiltroMtr) -> Result<LoteResult<Mtr>, OpenMTRError>;
    async fn criar_mtrs(...) -> Result<LoteResult<MtrCriado>, OpenMTRError>;
    async fn receber_mtrs(...) -> Result<LoteResult<MtrRecebido>, OpenMTRError>;
    async fn cancelar_mtrs(...) -> Result<LoteResult<MtrCancelado>, OpenMTRError>;
    async fn modificar_mtrs(...) -> Result<LoteResult<MtrModificado>, OpenMTRError>;
    async fn download_mtrs(...) -> Result<LoteResult<DownloadItem>, OpenMTRError>;
    async fn gerar_cdfs(...) -> Result<LoteResult<CdfEmitido>, OpenMTRError>;
    async fn consultar_cdfs(...) -> Result<LoteResult<Cdf>, OpenMTRError>;
    async fn download_cdfs(...) -> Result<LoteResult<DownloadItem>, OpenMTRError>;
    async fn listar_residuos(...) -> Result<LoteResult<ResiduoIbama>, OpenMTRError>;
    async fn listar_acondicionamentos(...) -> Result<LoteResult<Acondicionamento>, OpenMTRError>;
    async fn listar_classes(...) -> Result<LoteResult<ClasseResiduo>, OpenMTRError>;
    async fn listar_estados_fisicos(...) -> Result<LoteResult<EstadoFisico>, OpenMTRError>;
    async fn listar_tratamentos(...) -> Result<LoteResult<TratamentoResiduo>, OpenMTRError>;
    async fn listar_unidades(...) -> Result<LoteResult<UnidadeMedida>, OpenMTRError>;
}
```

## LoteResult

Operações em lote retornam `LoteResult<T>` que separa sucessos de erros:

```rust
LoteResult<T> {
    itens: Vec<ItemResult<T>>,
}

enum ItemResult<T> {
    Ok(T),
    Erro { indice: usize, problema: Problema },
}
```

## Implementação: SinirAdapter

`SinirAdapter` (840 linhas) implementa todos os 16 métodos para o SINIR Nacional:

- Usa `ctx.http.post::<Value, _>(path, &body, op)` para chamar endpoints SINIR
- Converte manualmente entre JSON (`serde_json::Value`) e tipos de domínio
- Funções auxiliares: `filtro_para_json()`, `json_para_mtr()`, `tem_erro()`, etc.

## Criando um novo adapter

```rust
struct MeuAdapter;

#[async_trait]
impl MtrAdapter for MeuAdapter {
    fn system(&self) -> System { System::MtrPe }
    fn nome(&self) -> &'static str { "MTR PE" }

    async fn consultar_mtrs(&self, ctx: &MtrCtx, filtro: &FiltroMtr) -> Result<LoteResult<Mtr>, OpenMTRError> {
        // 1. Montar requisição específica do sistema
        // 2. ctx.http.post() ou ctx.http.get()
        // 3. Converter resposta para domínio universal
        // 4. Retornar LoteResult
    }
    // ... +15 métodos
}
```
