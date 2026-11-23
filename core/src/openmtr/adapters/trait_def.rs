//! Contrato comum dos adapters de sistema (`SINIR`, `MTR-PE`, `SIGOR`).

use async_trait::async_trait;

use crate::openmtr::adapters::{LoteResult, MtrCtx};
use crate::openmtr::domain::{
    AceitarAlteracao, Acondicionamento, AlteracaoAceita, CancelarMtr, Cdf, CdfEmitido,
    ClasseResiduo, CriarMtr, CriarMtrComplementar, DownloadCdf, DownloadItem, EstadoFisico,
    FiltroCdf, FiltroMtr, GerarCdf, ModificarMtr, Mtr, MtrCancelado, MtrCriado, MtrModificado,
    MtrRecebido, ReceberMtr, ResiduoIbama, TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::open_mtr_auth::AuthParams;
use crate::openmtr::system::System;

/// Operações que um sistema estadual/federal expõe ao
/// [`crate::openmtr::open_mtr_client::OpenMtrClient`].
///
/// Cada adapter implementa apenas o que o seu sistema suporta; os métodos
/// restantes caem no default `not_supported`, mantendo o contrato uniforme.
#[async_trait]
pub trait MtrAdapter: Send + Sync {
    #[allow(
        dead_code,
        reason = "campo/helper mantido para compatibilidade da API e testes"
    )]
    /// Sistema que este adapter serve.
    fn system(&self) -> System;
    /// Nome legível do sistema, usado em logs e mensagens.
    fn nome(&self) -> &'static str;

    /// Autentica repetindo o token de integração (refresh) e devolve o access token.
    async fn autenticar_token(&self, ctx: &MtrCtx) -> Result<String, OpenMTRError>;

    /// Autentica com CPF/senha (login clássico) e devolve o access token.
    async fn autenticar_credenciais(
        &self,
        ctx: &MtrCtx,
        params: &AuthParams,
    ) -> Result<String, OpenMTRError>;

    /// Consulta MTRs por números ou período, devolvendo o lote com ordenação estável.
    async fn consultar_mtrs(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroMtr,
    ) -> Result<LoteResult<Mtr>, OpenMTRError>;

    /// Emite novos MTRs; cada item é independente no lote.
    async fn criar_mtrs(
        &self,
        ctx: &MtrCtx,
        mtrs: Vec<CriarMtr>,
    ) -> Result<LoteResult<MtrCriado>, OpenMTRError>;

    /// Registra o recebimento dos resíduos declarados em cada MTR.
    async fn receber_mtrs(
        &self,
        ctx: &MtrCtx,
        lotes: Vec<ReceberMtr>,
    ) -> Result<LoteResult<MtrRecebido>, OpenMTRError>;

    /// Cancela MTRs existentes.
    async fn cancelar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<CancelarMtr>,
    ) -> Result<LoteResult<MtrCancelado>, OpenMTRError>;

    /// Modifica dados de MTRs existentes.
    async fn modificar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<ModificarMtr>,
    ) -> Result<LoteResult<MtrModificado>, OpenMTRError>;

    /// Aceita ou recusa alterações propostas por outro parceiro.
    async fn aceitar_alteracoes(
        &self,
        ctx: &MtrCtx,
        _reqs: Vec<AceitarAlteracao>,
    ) -> Result<LoteResult<AlteracaoAceita>, OpenMTRError> {
        let _ = ctx;
        Err(OpenMTRError::not_supported("aceitar_alteracoes"))
    }

    /// Baixa o PDF de cada MTR solicitado.
    async fn download_mtrs(
        &self,
        ctx: &MtrCtx,
        nums: Vec<String>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError>;

    /// Emite CDFs a partir de MTRs recebidos.
    async fn gerar_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<GerarCdf>,
    ) -> Result<LoteResult<CdfEmitido>, OpenMTRError>;

    /// Consulta CDFs emitidos.
    async fn consultar_cdfs(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroCdf,
    ) -> Result<LoteResult<Cdf>, OpenMTRError>;

    /// Baixa o PDF de cada CDF solicitado.
    async fn download_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<DownloadCdf>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError>;

    /// Lista o catálogo de resíduos do IBAMA.
    async fn listar_residuos(&self, ctx: &MtrCtx)
    -> Result<LoteResult<ResiduoIbama>, OpenMTRError>;

    /// Lista os acondicionamentos aceitos pelo sistema.
    async fn listar_acondicionamentos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<Acondicionamento>, OpenMTRError>;

    /// Lista as classes de resíduo do sistema.
    async fn listar_classes(&self, ctx: &MtrCtx)
    -> Result<LoteResult<ClasseResiduo>, OpenMTRError>;

    /// Lista os estados físicos do resíduo.
    async fn listar_estados_fisicos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<EstadoFisico>, OpenMTRError>;

    /// Lista os tratamentos de resíduo disponíveis.
    async fn listar_tratamentos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<TratamentoResiduo>, OpenMTRError>;

    /// Lista as unidades de medida aceitas.
    async fn listar_unidades(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<UnidadeMedida>, OpenMTRError>;

    // ─── SIGOR-specific endpoints (with default fallbacks) ───

    /// Lista classes compatíveis com um código IBAMA (específico do SIGOR).
    async fn listar_classes_por_residuo(
        &self,
        _ctx: &MtrCtx,
        _codigo_ibama: &str,
    ) -> Result<LoteResult<ClasseResiduo>, OpenMTRError> {
        Err(OpenMTRError::not_supported("listar_classes_por_residuo"))
    }

    /// Lista acondicionamentos compatíveis com um estado físico (específico do SIGOR).
    async fn listar_acondicionamentos_por_estado_fisico(
        &self,
        _ctx: &MtrCtx,
        _codigo_estado: u32,
    ) -> Result<LoteResult<Acondicionamento>, OpenMTRError> {
        Err(OpenMTRError::not_supported(
            "listar_acondicionamentos_por_estado_fisico",
        ))
    }

    /// Consulta um MTR por código de referência da unidade (específico do SIGOR).
    async fn consultar_mtr_por_codigo_referencia(
        &self,
        _ctx: &MtrCtx,
        _unidade: u32,
        _codigo: &str,
    ) -> Result<Option<Mtr>, OpenMTRError> {
        Err(OpenMTRError::not_supported(
            "consultar_mtr_por_codigo_referencia",
        ))
    }

    /// Cria um manifesto complementar a partir de um MTR existente.
    async fn criar_manifesto_complementar(
        &self,
        _ctx: &MtrCtx,
        _req: CriarMtrComplementar,
    ) -> Result<MtrCriado, OpenMTRError> {
        Err(OpenMTRError::not_supported("criar_manifesto_complementar"))
    }
}
