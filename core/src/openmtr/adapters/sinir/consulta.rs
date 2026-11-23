//! Operações SINIR: consulta.

use serde_json::Value;
use tracing::{info, instrument, warn};

use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx};
use crate::openmtr::domain::{FiltroMtr, Mtr};
use crate::openmtr::error::OpenMTRError;

use super::SinirAdapter;
use super::json_escrita::{escrita_filtro_json, escrita_lista_manifesto_path};
use super::json_listas::{extrair_objeto_resposta, tem_erro};
use super::json_mtr::{json_mtr, json_mtr_detalhe};

impl SinirAdapter {
    #[instrument(name = "sinir.consultar_mtrs", skip_all)]
    /// Executa a operação SINIR `consultar_mtrs`.
    pub(super) async fn consultar_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroMtr,
    ) -> Result<LoteResult<Mtr>, OpenMTRError> {
        // Se há números específicos, usa o endpoint de detalhe
        if let Some(ref nums) = filtro.numeros
            && !nums.is_empty()
        {
            return self.consultar_mtrs_por_numeros_impl(ctx, nums).await;
        }

        // Caso contrário, busca por período + tipo de parceiro (paginação via query string)
        let body = escrita_filtro_json(filtro, ctx.today());
        let path = escrita_lista_manifesto_path(filtro);
        let response: Value = ctx.http.post(&path, &body, "consultar_mtrs").await?;

        if tem_erro(&response) {
            warn!(
                mensagem = ?response.get("mensagem"),
                "SINIR retornou erro em consultar_mtrs"
            );
            return Ok(LoteResult::from_vec(Vec::new()));
        }

        let total_reg = response.get("totalRecords").and_then(Value::as_i64);

        let items = response
            .get("objetoResposta")
            .and_then(Value::as_array)
            .map_or_default(Vec::as_slice);

        // Se detalhado, extrai só os números e busca tudo via retornaManifesto
        if filtro.detalhado && !items.is_empty() {
            let numeros: Vec<String> = items
                .iter()
                .filter_map(|v| v.get("manNumero").and_then(|s| s.as_str()))
                .map(String::from)
                .collect();
            let mut lote = self.consultar_mtrs_por_numeros_impl(ctx, &numeros).await?;
            lote.total = total_reg;
            info!(total = %numeros.len(), detalhados = %lote.total_ok(), "MTRs consultados com detalhes");
            return Ok(lote);
        }

        let mtrs: Vec<Mtr> = items.iter().map(json_mtr).collect();
        info!(total = %mtrs.len(), "MTRs consultados");

        match total_reg {
            Some(t) => Ok(LoteResult::from_vec_com_total(mtrs, t)),
            None => Ok(LoteResult::from_vec(mtrs)),
        }
    }
    /// Consulta MTRs específicos via GET /retornaManifesto/{numero}
    /// Consulta MTRs específicos via GET /retornaManifesto/{numero}.
    pub(super) async fn consultar_mtrs_por_numeros_impl(
        &self,
        ctx: &MtrCtx,
        numeros: &[String],
    ) -> Result<LoteResult<Mtr>, OpenMTRError> {
        let mut itens = Vec::with_capacity(numeros.len());

        for (indice, numero) in numeros.iter().enumerate() {
            let path = format!("/retornaManifesto/{numero}");
            match ctx.http.get::<Value>(&path, "consultar_mtr_detalhe").await {
                Ok(response) => {
                    if tem_erro(&response) {
                        let msg = response
                            .get("mensagem")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Erro ao consultar MTR");
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::not_found(msg),
                        });
                        continue;
                    }
                    match extrair_objeto_resposta(&response) {
                        Some(detail) => {
                            let mtr = json_mtr_detalhe(detail);
                            itens.push(ItemResult::Ok(mtr));
                        }
                        None => {
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::not_found("Resposta vazia do SINIR"),
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!(numero = %numero, erro = %e, "Falha ao consultar MTR detalhe");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        info!(
            total = %itens.len(),
            sucessos = %itens.iter().filter(|i| matches!(i, ItemResult::Ok(_))).count(),
            "MTRs consultados por número"
        );
        Ok(LoteResult::new(itens))
    }
}
