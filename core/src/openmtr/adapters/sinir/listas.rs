//! Operações SINIR: listas.

use serde_json::Value;
use tracing::{info, instrument};

use crate::openmtr::adapters::{LoteResult, MtrCtx};
use crate::openmtr::domain::{
    Acondicionamento, ClasseResiduo, EstadoFisico, ResiduoIbama, TratamentoResiduo, UnidadeMedida,
};
use crate::openmtr::error::OpenMTRError;

use super::SinirAdapter;
use super::json_listas::{
    lista_acondicionamento, lista_classe, lista_estado_fisico, lista_residuo_ibama,
    lista_tratamento, lista_unidade,
};

impl SinirAdapter {
    #[instrument(name = "sinir.listar_residuos", skip_all)]
    /// Executa a operação SINIR `listar_residuos`.
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
        let domain: Vec<ResiduoIbama> = items.iter().map(lista_residuo_ibama).collect();

        info!(total = %domain.len(), "Resíduos listados");
        Ok(LoteResult::from_vec(domain))
    }

    #[instrument(name = "sinir.listar_acondicionamentos", skip_all)]
    /// Executa a operação SINIR `listar_acondicionamentos`.
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
        let domain: Vec<Acondicionamento> = items.iter().map(lista_acondicionamento).collect();

        info!(total = %domain.len(), "Acondicionamentos listados");
        Ok(LoteResult::from_vec(domain))
    }

    #[instrument(name = "sinir.listar_classes", skip_all)]
    /// Executa a operação SINIR `listar_classes`.
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
        let domain: Vec<ClasseResiduo> = items.iter().map(lista_classe).collect();

        info!(total = %domain.len(), "Classes listadas");
        Ok(LoteResult::from_vec(domain))
    }

    #[instrument(name = "sinir.listar_estados_fisicos", skip_all)]
    /// Executa a operação SINIR `listar_estados_fisicos`.
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
        let domain: Vec<EstadoFisico> = items.iter().map(lista_estado_fisico).collect();

        info!(total = %domain.len(), "Estados físicos listados");
        Ok(LoteResult::from_vec(domain))
    }

    #[instrument(name = "sinir.listar_tratamentos", skip_all)]
    /// Executa a operação SINIR `listar_tratamentos`.
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
        let domain: Vec<TratamentoResiduo> = items.iter().map(lista_tratamento).collect();

        info!(total = %domain.len(), "Tratamentos listados");
        Ok(LoteResult::from_vec(domain))
    }

    #[instrument(name = "sinir.listar_unidades", skip_all)]
    /// Executa a operação SINIR `listar_unidades`.
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
        let domain: Vec<UnidadeMedida> = items.iter().map(lista_unidade).collect();

        info!(total = %domain.len(), "Unidades listadas");
        Ok(LoteResult::from_vec(domain))
    }
}
