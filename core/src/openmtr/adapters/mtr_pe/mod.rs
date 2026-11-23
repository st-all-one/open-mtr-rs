#![allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::indexing_slicing,
    reason = "adapters legados normalizam números/índices de JSON; a extração tipada entra na revitalização"
)]

//! Adapter MTR Pernambuco: coordena autenticação, consultas, escrita, downloads e listas.
//!
//! Cada submódulo contém os corpos das operações; este arquivo mantém apenas a
//! implementação de [`MtrAdapter`] com delegações finas, já que impls de trait
//! não podem ser divididas entre arquivos.

use async_trait::async_trait;
use tracing::instrument;

use crate::openmtr::adapters::{LoteResult, MtrAdapter, MtrCtx};
use crate::openmtr::domain::{
    AceitarAlteracao, Acondicionamento, AlteracaoAceita, CancelarMtr, Cdf, CdfEmitido,
    ClasseResiduo, CriarMtr, DownloadCdf, DownloadItem, EstadoFisico, FiltroCdf, FiltroMtr,
    GerarCdf, ModificarMtr, Mtr, MtrCancelado, MtrCriado, MtrModificado, MtrRecebido, ReceberMtr,
    ResiduoIbama, TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::system::System;

mod alteracoes;
mod autenticacao;
mod cdfs;
mod consulta;
mod escrita;
mod listas;
mod periodo;
mod tipos;

/// Adapter para o sistema MTR Pernambuco.
pub(crate) struct MtrPeAdapter;

#[async_trait]
impl MtrAdapter for MtrPeAdapter {
    fn system(&self) -> System {
        System::MtrPe
    }

    fn nome(&self) -> &'static str {
        "MTR Pernambuco"
    }

    async fn autenticar_token(&self, ctx: &MtrCtx) -> Result<String, OpenMTRError> {
        Self::autenticar_token_impl(ctx)
    }

    #[instrument(name = "mtr_pe.autenticar_credenciais", skip_all)]
    async fn autenticar_credenciais(
        &self,
        ctx: &MtrCtx,
        params: &crate::openmtr::open_mtr_auth::AuthParams,
    ) -> Result<String, OpenMTRError> {
        self.autenticar_credenciais_impl(ctx, params).await
    }

    #[instrument(name = "mtr_pe.consultar_mtrs", skip_all)]
    async fn consultar_mtrs(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroMtr,
    ) -> Result<LoteResult<Mtr>, OpenMTRError> {
        self.consultar_mtrs_impl(ctx, filtro).await
    }

    #[instrument(name = "mtr_pe.criar_mtrs", skip_all)]
    async fn criar_mtrs(
        &self,
        ctx: &MtrCtx,
        mtrs: Vec<CriarMtr>,
    ) -> Result<LoteResult<MtrCriado>, OpenMTRError> {
        self.criar_mtrs_impl(ctx, mtrs).await
    }

    #[instrument(name = "mtr_pe.receber_mtrs", skip_all)]
    async fn receber_mtrs(
        &self,
        ctx: &MtrCtx,
        lotes: Vec<ReceberMtr>,
    ) -> Result<LoteResult<MtrRecebido>, OpenMTRError> {
        self.receber_mtrs_impl(ctx, lotes).await
    }

    #[instrument(name = "mtr_pe.cancelar_mtrs", skip_all)]
    async fn cancelar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<CancelarMtr>,
    ) -> Result<LoteResult<MtrCancelado>, OpenMTRError> {
        self.cancelar_mtrs_impl(ctx, reqs).await
    }

    #[instrument(name = "mtr_pe.modificar_mtrs", skip_all)]
    async fn modificar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<ModificarMtr>,
    ) -> Result<LoteResult<MtrModificado>, OpenMTRError> {
        self.modificar_mtrs_impl(ctx, reqs).await
    }

    #[instrument(name = "mtr_pe.aceitar_alteracoes", skip_all)]
    async fn aceitar_alteracoes(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<AceitarAlteracao>,
    ) -> Result<LoteResult<AlteracaoAceita>, OpenMTRError> {
        self.aceitar_alteracoes_impl(ctx, reqs).await
    }

    #[instrument(name = "mtr_pe.download_mtrs", skip_all)]
    async fn download_mtrs(
        &self,
        ctx: &MtrCtx,
        nums: Vec<String>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        self.download_mtrs_impl(ctx, nums).await
    }

    #[instrument(name = "mtr_pe.gerar_cdfs", skip_all)]
    async fn gerar_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<GerarCdf>,
    ) -> Result<LoteResult<CdfEmitido>, OpenMTRError> {
        self.gerar_cdfs_impl(ctx, reqs).await
    }

    #[instrument(name = "mtr_pe.consultar_cdfs", skip_all)]
    async fn consultar_cdfs(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroCdf,
    ) -> Result<LoteResult<Cdf>, OpenMTRError> {
        self.consultar_cdfs_impl(ctx, filtro).await
    }

    #[instrument(name = "mtr_pe.download_cdfs", skip_all)]
    async fn download_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<DownloadCdf>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        self.download_cdfs_impl(ctx, reqs).await
    }

    #[instrument(name = "mtr_pe.listar_residuos", skip_all)]
    async fn listar_residuos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ResiduoIbama>, OpenMTRError> {
        self.listar_residuos_impl(ctx).await
    }

    #[instrument(name = "mtr_pe.listar_acondicionamentos", skip_all)]
    async fn listar_acondicionamentos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<Acondicionamento>, OpenMTRError> {
        self.listar_acondicionamentos_impl(ctx).await
    }

    #[instrument(name = "mtr_pe.listar_classes", skip_all)]
    async fn listar_classes(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ClasseResiduo>, OpenMTRError> {
        self.listar_classes_impl(ctx).await
    }

    #[instrument(name = "mtr_pe.listar_estados_fisicos", skip_all)]
    async fn listar_estados_fisicos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<EstadoFisico>, OpenMTRError> {
        self.listar_estados_fisicos_impl(ctx).await
    }

    #[instrument(name = "mtr_pe.listar_tratamentos", skip_all)]
    async fn listar_tratamentos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<TratamentoResiduo>, OpenMTRError> {
        self.listar_tratamentos_impl(ctx).await
    }

    #[instrument(name = "mtr_pe.listar_unidades", skip_all)]
    async fn listar_unidades(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<UnidadeMedida>, OpenMTRError> {
        self.listar_unidades_impl(ctx).await
    }
}
