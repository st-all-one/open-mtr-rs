#![allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::indexing_slicing,
    reason = "adapters legados normalizam números/índices de JSON; a extração tipada entra na revitalização"
)]

//! Adapter SINIR Nacional: coordena autenticação, consultas, escrita, downloads e listas.
//!
//! Cada submódulo contém os corpos das operações; este arquivo mantém apenas a
//! implementação de [`MtrAdapter`] com delegações finas, já que impls de trait
//! não podem ser divididas entre arquivos.

use crate::openmtr::adapters::{LoteResult, MtrAdapter, MtrCtx};
use crate::openmtr::domain::{
    AceitarAlteracao, Acondicionamento, AlteracaoAceita, CancelarMtr, Cdf, CdfEmitido,
    ClasseResiduo, CriarMtr, DownloadCdf, DownloadItem, EstadoFisico, FiltroCdf, FiltroMtr,
    GerarCdf, ModificarMtr, Mtr, MtrCancelado, MtrCriado, MtrModificado, MtrRecebido, ReceberMtr,
    ResiduoIbama, TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::system::System;
use async_trait::async_trait;

mod alteracao;
mod auth;
mod consulta;
mod download;
mod escrita;
mod json_escrita;
mod json_listas;
mod json_mtr;
mod json_residuos;
mod listas;
#[cfg(test)]
mod testes;

/// Adapter para o sistema SINIR Nacional.
pub(crate) struct SinirAdapter;

#[async_trait]
impl MtrAdapter for SinirAdapter {
    fn system(&self) -> System {
        System::Sinir
    }

    fn nome(&self) -> &'static str {
        "SINIR Nacional"
    }

    async fn autenticar_token(&self, ctx: &MtrCtx) -> Result<String, OpenMTRError> {
        self.autenticar_token_impl(ctx).await
    }

    async fn autenticar_credenciais(
        &self,
        ctx: &MtrCtx,
        params: &crate::openmtr::open_mtr_auth::AuthParams,
    ) -> Result<String, OpenMTRError> {
        Self::autenticar_credenciais_impl(ctx, params)
    }

    async fn consultar_mtrs(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroMtr,
    ) -> Result<LoteResult<Mtr>, OpenMTRError> {
        self.consultar_mtrs_impl(ctx, filtro).await
    }

    async fn criar_mtrs(
        &self,
        ctx: &MtrCtx,
        mtrs: Vec<CriarMtr>,
    ) -> Result<LoteResult<MtrCriado>, OpenMTRError> {
        self.criar_mtrs_impl(ctx, mtrs).await
    }

    async fn receber_mtrs(
        &self,
        ctx: &MtrCtx,
        lotes: Vec<ReceberMtr>,
    ) -> Result<LoteResult<MtrRecebido>, OpenMTRError> {
        self.receber_mtrs_impl(ctx, lotes).await
    }

    async fn cancelar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<CancelarMtr>,
    ) -> Result<LoteResult<MtrCancelado>, OpenMTRError> {
        self.cancelar_mtrs_impl(ctx, reqs).await
    }

    async fn modificar_mtrs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<ModificarMtr>,
    ) -> Result<LoteResult<MtrModificado>, OpenMTRError> {
        self.modificar_mtrs_impl(ctx, reqs).await
    }

    async fn aceitar_alteracoes(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<AceitarAlteracao>,
    ) -> Result<LoteResult<AlteracaoAceita>, OpenMTRError> {
        self.aceitar_alteracoes_impl(ctx, reqs).await
    }

    async fn download_mtrs(
        &self,
        ctx: &MtrCtx,
        nums: Vec<String>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        self.download_mtrs_impl(ctx, nums).await
    }

    async fn gerar_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<GerarCdf>,
    ) -> Result<LoteResult<CdfEmitido>, OpenMTRError> {
        self.gerar_cdfs_impl(ctx, reqs).await
    }

    async fn consultar_cdfs(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroCdf,
    ) -> Result<LoteResult<Cdf>, OpenMTRError> {
        Self::consultar_cdfs_impl(ctx, filtro)
    }

    async fn download_cdfs(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<DownloadCdf>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        self.download_cdfs_impl(ctx, reqs).await
    }

    async fn listar_residuos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ResiduoIbama>, OpenMTRError> {
        self.listar_residuos_impl(ctx).await
    }

    async fn listar_acondicionamentos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<Acondicionamento>, OpenMTRError> {
        self.listar_acondicionamentos_impl(ctx).await
    }

    async fn listar_classes(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ClasseResiduo>, OpenMTRError> {
        self.listar_classes_impl(ctx).await
    }

    async fn listar_estados_fisicos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<EstadoFisico>, OpenMTRError> {
        self.listar_estados_fisicos_impl(ctx).await
    }

    async fn listar_tratamentos(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<TratamentoResiduo>, OpenMTRError> {
        self.listar_tratamentos_impl(ctx).await
    }

    async fn listar_unidades(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<UnidadeMedida>, OpenMTRError> {
        self.listar_unidades_impl(ctx).await
    }
}
