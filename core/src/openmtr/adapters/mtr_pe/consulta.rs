//! Consulta de manifestos no MTR-PE.

use serde_json::{Value, json};
use tracing::instrument;

use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx};
use crate::openmtr::domain::{FiltroMtr, Mtr};
use crate::openmtr::error::OpenMTRError;

use super::MtrPeAdapter;
use super::tipos::{json_para_mtr, manifesto_recuperavel, retorno_eh_sucesso};

impl MtrPeAdapter {
    #[instrument(name = "mtr_pe.consultar_mtrs", skip_all)]
    /// Consulta MTRs por número ou delega a consulta por período.
    pub(super) async fn consultar_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroMtr,
    ) -> Result<LoteResult<Mtr>, OpenMTRError> {
        if let Some(ref nums) = filtro.numeros
            && !nums.is_empty()
        {
            let mut itens = Vec::with_capacity(nums.len());
            for (indice, numero) in nums.iter().enumerate() {
                let path = format!("/retornaManifesto/{numero}");
                match ctx
                    .http
                    .post::<Value, _>(&path, &json!({}), "consultar_mtr")
                    .await
                {
                    Ok(resp) => {
                        if retorno_eh_sucesso(&resp) || manifesto_recuperavel(&resp) {
                            itens.push(ItemResult::Ok(json_para_mtr(&resp, numero)));
                        } else {
                            let msg = resp
                                .get("retorno")
                                .and_then(|s| s.as_str())
                                .unwrap_or("Erro");
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::not_found(msg),
                            });
                        }
                    }
                    Err(e) => {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::api_error(502, e.to_string()),
                        });
                    }
                }
            }
            return Ok(LoteResult::new(itens));
        }

        // Consulta por período: o MTR-PE não lista manifestos estruturados —
        // retornaListaCodigoBarasManifesto devolve apenas códigos de barras
        // (janela máxima de 30 dias). O helper coleta os barcodes e busca o
        // detalhe de cada um via /retornaManifesto/{barcode}.
        if let Some(ref periodo) = filtro.periodo {
            return self.consultar_mtrs_por_periodo_impl(ctx, periodo).await;
        }

        Err(OpenMTRError::validation(
            "MTR-PE: informe 'numeros' ou 'periodo' no FiltroMtr",
        ))
    }
}
