//! Operações SINIR: cancelamento, modificação e aceite de alterações.

use serde_json::{Value, json};
use tracing::{info, instrument, warn};

use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx};
use crate::openmtr::domain::{
    AceitarAlteracao, AlteracaoAceita, CancelarMtr, ModificacaoTipo, ModificarMtr, MtrCancelado,
    MtrModificado,
};
use crate::openmtr::error::OpenMTRError;

use super::SinirAdapter;
use super::json_listas::{extrair_objeto_resposta, tem_erro};

impl SinirAdapter {
    #[instrument(name = "sinir.cancelar_mtrs", skip_all)]
    /// Executa a operação SINIR `cancelar_mtrs`.
    pub(super) async fn cancelar_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<CancelarMtr>,
    ) -> Result<LoteResult<MtrCancelado>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
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
                            .unwrap_or("Erro no cancelamento");
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(msg),
                        });
                        continue;
                    }

                    match extrair_objeto_resposta(&resp) {
                        Some(cancel) => {
                            let status_str = cancel
                                .get("status")
                                .and_then(|s| s.as_str())
                                .unwrap_or_default()
                                .to_lowercase();
                            if status_str == "cancelado" {
                                itens.push(ItemResult::Ok(MtrCancelado {
                                    numero: cancel
                                        .get("manNumero")
                                        .and_then(|s| s.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    status: cancel
                                        .get("status")
                                        .and_then(|s| s.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    data_cancelamento: cancel
                                        .get("dataCancelamento")
                                        .and_then(|s| s.as_str())
                                        .map(String::from),
                                }));
                            } else {
                                let stat = cancel
                                    .get("status")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("unknown");
                                itens.push(ItemResult::Erro {
                                    indice,
                                    problema: OpenMTRError::api_error(
                                        400,
                                        format!("Status inesperado: {stat}"),
                                    ),
                                });
                            }
                        }
                        None => {
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::validation(
                                    "Resposta vazia no cancelamento",
                                ),
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!(erro = %e, "Falha ao cancelar MTR");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        info!(total = %itens.len(), "MTRs cancelados");
        Ok(LoteResult::new(itens))
    }
    #[instrument(name = "sinir.modificar_mtrs", skip_all)]
    /// Executa a operação SINIR `modificar_mtrs`.
    pub(super) async fn modificar_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<ModificarMtr>,
    ) -> Result<LoteResult<MtrModificado>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            match req.tipo {
                ModificacaoTipo::Recebimento => {
                    let justificativa = req
                        .justificativa
                        .unwrap_or_else(|| "Alteração de recebimento".into());
                    let novos = req.novos_residuos.unwrap_or_default();

                    let body = json!({
                        "manNumero": req.numero,
                        "justificativa": justificativa,
                        "listaResiduosAlterados": novos.iter().map(|r| json!({
                            "resCodigoIbama": r.codigo_ibama,
                            "novaQuantidade": r.quantidade_recebida,
                            "uniCodigo": r.unidade,
                            "observacoes": Option::<String>::None,
                        })).collect::<Vec<_>>(),
                    });

                    match ctx
                        .http
                        .post::<Value, _>("/alteraRecebimento", &body, "modificar_mtr")
                        .await
                    {
                        Ok(_json) => {
                            itens.push(ItemResult::Ok(MtrModificado {
                                numero: req.numero,
                                modificado: true,
                                mensagem: "Solicitação de alteração enviada".into(),
                            }));
                        }
                        Err(e) => {
                            warn!(erro = %e, "Falha ao modificar MTR");
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::api_error(502, e.to_string()),
                            });
                        }
                    }
                }
                _ => {
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::validation(format!(
                            "Tipo de modificação não suportado no SINIR: {:?}",
                            req.tipo
                        )),
                    });
                }
            }
        }

        info!(total = %itens.len(), "MTRs modificados");
        Ok(LoteResult::new(itens))
    }
    #[instrument(name = "sinir.aceitar_alteracoes", skip_all)]
    /// Executa a operação SINIR `aceitar_alteracoes`.
    pub(super) async fn aceitar_alteracoes_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<AceitarAlteracao>,
    ) -> Result<LoteResult<AlteracaoAceita>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let body = json!({
                "manNumero": req.numero,
                "aceite": req.aceitar,
            });

            match ctx
                .http
                .post::<Value, _>("/aceiteAlteracaoRecebimento", &body, "aceitar_alteracao")
                .await
            {
                Ok(resp) => {
                    if tem_erro(&resp) {
                        let msg = resp
                            .get("mensagem")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Erro no aceite");
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(msg),
                        });
                        continue;
                    }
                    let aceita = resp
                        .get("alteracaoAceita")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let msg = resp
                        .get("mensagem")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default()
                        .to_string();
                    itens.push(ItemResult::Ok(AlteracaoAceita {
                        numero: req.numero,
                        aceita,
                        mensagem: msg,
                    }));
                }
                Err(e) => {
                    warn!(erro = %e, "Falha ao aceitar alteração no SINIR");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        info!(total = %itens.len(), "Alterações aceitas/recusadas");
        Ok(LoteResult::new(itens))
    }
}
