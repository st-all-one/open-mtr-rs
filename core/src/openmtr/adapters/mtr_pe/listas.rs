//! Listas de domínio do MTR-PE: resíduos, classes, unidades etc.

use serde_json::{Value, json};
use tracing::instrument;

use crate::openmtr::adapters::{LoteResult, MtrCtx};
use crate::openmtr::domain::{
    Acondicionamento, ClasseResiduo, EstadoFisico, ResiduoIbama, TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;

use super::MtrPeAdapter;
use super::tipos::{
    json_para_acondicionamento, json_para_classe, json_para_estado_fisico, json_para_residuo_ibama,
    json_para_tratamento, json_para_unidade,
};

impl MtrPeAdapter {
    #[instrument(name = "mtr_pe.listar_residuos", skip_all)]
    /// Lista resíduos IBAMA disponíveis.
    pub(super) async fn listar_residuos_impl(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ResiduoIbama>, OpenMTRError> {
        let response: Value = ctx
            .http
            .post("/retornaListaResiduo", &json!({}), "listar_residuos")
            .await?;
        let items = response.as_array().map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_residuo_ibama).collect(),
        ))
    }

    #[instrument(name = "mtr_pe.listar_acondicionamentos", skip_all)]
    /// Lista acondicionamentos disponíveis.
    pub(super) async fn listar_acondicionamentos_impl(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<Acondicionamento>, OpenMTRError> {
        let response: Value = ctx
            .http
            .post(
                "/retornaListaAcondicionamento",
                &json!({}),
                "listar_acondicionamentos",
            )
            .await?;
        let items = response.as_array().map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_acondicionamento).collect(),
        ))
    }

    #[instrument(name = "mtr_pe.listar_classes", skip_all)]
    /// Lista classes de resíduo disponíveis.
    pub(super) async fn listar_classes_impl(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ClasseResiduo>, OpenMTRError> {
        let response: Value = ctx
            .http
            .post("/retornaListaClasse", &json!({}), "listar_classes")
            .await?;
        let items = response.as_array().map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_classe).collect(),
        ))
    }

    #[instrument(name = "mtr_pe.listar_estados_fisicos", skip_all)]
    /// Lista estados físicos disponíveis.
    pub(super) async fn listar_estados_fisicos_impl(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<EstadoFisico>, OpenMTRError> {
        let response: Value = ctx
            .http
            .post(
                "/retornaListaEstadoFisico",
                &json!({}),
                "listar_estados_fisicos",
            )
            .await?;
        let items = response.as_array().map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_estado_fisico).collect(),
        ))
    }

    #[instrument(name = "mtr_pe.listar_tratamentos", skip_all)]
    /// Lista tratamentos de resíduo disponíveis.
    pub(super) async fn listar_tratamentos_impl(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<TratamentoResiduo>, OpenMTRError> {
        let response: Value = ctx
            .http
            .post("/retornaListaTecnologia", &json!({}), "listar_tratamentos")
            .await?;
        let items = response.as_array().map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_tratamento).collect(),
        ))
    }

    #[instrument(name = "mtr_pe.listar_unidades", skip_all)]
    /// Lista unidades de medida disponíveis.
    pub(super) async fn listar_unidades_impl(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<UnidadeMedida>, OpenMTRError> {
        let response: Value = ctx
            .http
            .post("/retornaListaUnidade", &json!({}), "listar_unidades")
            .await?;
        let items = response.as_array().map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_unidade).collect(),
        ))
    }
}
