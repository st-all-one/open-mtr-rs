//! Cancelamento de manifestos no SIGOR.

use serde_json::{Value, json};
use tracing::instrument;

use super::SigorAdapter;
use super::tipos::tem_erro;
use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx};
use crate::openmtr::domain::{CancelarMtr, MtrCancelado};
use crate::openmtr::error::OpenMTRError;

impl SigorAdapter {
    /// Cancela manifestos: via individual quando há um só, em lote caso contrário.
    #[instrument(name = "sigor.cancelar_mtrs", skip_all)]
    pub(super) async fn cancelar_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<CancelarMtr>,
    ) -> Result<LoteResult<MtrCancelado>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        if reqs.len() == 1 {
            // Single cancel via /cancelarManifesto
            let req = &reqs[0];
            let body = json!({ "manNumero": req.numero, "justificativa": req.justificativa });
            match ctx
                .http
                .post::<Value, _>("/cancelarManifesto", &body, "cancelar_mtr")
                .await
            {
                Ok(resp) => {
                    if tem_erro(&resp) {
                        let msg = resp
                            .get("mensagem")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Erro");
                        itens.push(ItemResult::Erro {
                            indice: 0,
                            problema: OpenMTRError::validation(msg),
                        });
                    } else {
                        itens.push(ItemResult::Ok(MtrCancelado {
                            numero: req.numero.clone(),
                            status: "Cancelado".into(),
                            data_cancelamento: None,
                        }));
                    }
                }
                Err(e) => {
                    itens.push(ItemResult::Erro {
                        indice: 0,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        } else {
            // Batch cancel via /cancelarManifestoLote
            let batch: Vec<Value> = reqs
                .iter()
                .map(|r| json!({ "manNumero": r.numero, "justificativa": r.justificativa }))
                .collect();
            let body = json!(batch);
            match ctx
                .http
                .post::<Value, _>("/cancelarManifestoLote", &body, "cancelar_mtrs_lote")
                .await
            {
                Ok(resp) => {
                    let results = resp
                        .get("objetoResposta")
                        .and_then(Value::as_array)
                        .map_or_default(Vec::as_slice);
                    for (i, dto) in results.iter().enumerate() {
                        let erro = dto.get("erro").and_then(Value::as_bool).unwrap_or(true);
                        let man_num = dto
                            .get("manNumero")
                            .and_then(|s| s.as_str())
                            .unwrap_or_default();
                        if erro {
                            let msg = dto
                                .get("mensgaemRetorno")
                                .and_then(|s| s.as_str())
                                .unwrap_or("Erro");
                            itens.push(ItemResult::Erro {
                                indice: i,
                                problema: OpenMTRError::validation(msg),
                            });
                        } else {
                            itens.push(ItemResult::Ok(MtrCancelado {
                                numero: man_num.to_string(),
                                status: "Cancelado".into(),
                                data_cancelamento: None,
                            }));
                        }
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    for i in 0..reqs.len() {
                        itens.push(ItemResult::Erro {
                            indice: i,
                            problema: OpenMTRError::api_error(502, msg.clone()),
                        });
                    }
                }
            }
        }

        Ok(LoteResult::new(itens))
    }
}
