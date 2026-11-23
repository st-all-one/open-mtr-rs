//! Listagens de tabelas auxiliares do SIGOR (via GET).

use serde_json::Value;
use tracing::instrument;

use super::SigorAdapter;
use super::tipos::{
    json_para_acondicionamento, json_para_classe, json_para_estado_fisico, json_para_residuo_ibama,
    json_para_tratamento, json_para_unidade,
};
use crate::openmtr::adapters::{LoteResult, MtrCtx};
use crate::openmtr::domain::{
    Acondicionamento, ClasseResiduo, EstadoFisico, ResiduoIbama, TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;

impl SigorAdapter {
    /// Lista os resíduos IBAMA disponíveis.
    #[instrument(name = "sigor.listar_residuos", skip_all)]
    pub(super) async fn listar_residuos_impl(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ResiduoIbama>, OpenMTRError> {
        let response: Value = ctx
            .http
            .get("/retornaListaResiduo", "listar_residuos")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_residuo_ibama).collect(),
        ))
    }

    /// Lista os acondicionamentos disponíveis.
    #[instrument(name = "sigor.listar_acondicionamentos", skip_all)]
    pub(super) async fn listar_acondicionamentos_impl(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<Acondicionamento>, OpenMTRError> {
        let response: Value = ctx
            .http
            .get("/retornaListaAcondicionamento", "listar_acondicionamentos")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_acondicionamento).collect(),
        ))
    }

    /// Lista as classes de resíduo disponíveis.
    #[instrument(name = "sigor.listar_classes", skip_all)]
    pub(super) async fn listar_classes_impl(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<ClasseResiduo>, OpenMTRError> {
        let response: Value = ctx
            .http
            .get("/retornaListaClasse", "listar_classes")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_classe).collect(),
        ))
    }

    /// Lista os estados físicos disponíveis.
    #[instrument(name = "sigor.listar_estados_fisicos", skip_all)]
    pub(super) async fn listar_estados_fisicos_impl(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<EstadoFisico>, OpenMTRError> {
        let response: Value = ctx
            .http
            .get("/retornaListaEstadoFisico", "listar_estados_fisicos")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_estado_fisico).collect(),
        ))
    }

    /// Lista os tratamentos disponíveis.
    #[instrument(name = "sigor.listar_tratamentos", skip_all)]
    pub(super) async fn listar_tratamentos_impl(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<TratamentoResiduo>, OpenMTRError> {
        let response: Value = ctx
            .http
            .get("/retornaListaTratamento", "listar_tratamentos")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_tratamento).collect(),
        ))
    }

    /// Lista as unidades de medida disponíveis.
    #[instrument(name = "sigor.listar_unidades", skip_all)]
    pub(super) async fn listar_unidades_impl(
        &self,
        ctx: &MtrCtx,
    ) -> Result<LoteResult<UnidadeMedida>, OpenMTRError> {
        let response: Value = ctx
            .http
            .get("/retornaListaUnidade", "listar_unidades")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_unidade).collect(),
        ))
    }

    /// Lista as classes associadas a um resíduo IBAMA.
    #[instrument(name = "sigor.listar_classes_por_residuo", skip_all)]
    pub(super) async fn listar_classes_por_residuo_impl(
        &self,
        ctx: &MtrCtx,
        codigo_ibama: &str,
    ) -> Result<LoteResult<ClasseResiduo>, OpenMTRError> {
        let path = format!("/retornaListaClassePorResiduo/{codigo_ibama}");
        let response: Value = ctx.http.get(&path, "listar_classes_por_residuo").await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_classe).collect(),
        ))
    }

    /// Lista os acondicionamentos associados a um estado físico.
    #[instrument(name = "sigor.listar_acondicionamentos_por_estado_fisico", skip_all)]
    pub(super) async fn listar_acondicionamentos_por_estado_fisico_impl(
        &self,
        ctx: &MtrCtx,
        codigo_estado: u32,
    ) -> Result<LoteResult<Acondicionamento>, OpenMTRError> {
        let path = format!("/retornaListaAcondicionamentoPorEstadoFisico/{codigo_estado}");
        let response: Value = ctx
            .http
            .get(&path, "listar_acondicionamentos_por_estado")
            .await?;
        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map_or_default(Vec::as_slice);
        Ok(LoteResult::from_vec(
            items.iter().map(json_para_acondicionamento).collect(),
        ))
    }
}
