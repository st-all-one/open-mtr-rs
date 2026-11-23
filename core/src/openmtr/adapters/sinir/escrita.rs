//! Operações SINIR: criação e recebimento de MTRs.

use serde_json::Value;
use tracing::{info, instrument, warn};

use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx};
use crate::openmtr::domain::{CriarMtr, MtrCriado, MtrRecebido, ReceberMtr};
use crate::openmtr::error::OpenMTRError;

use super::SinirAdapter;
use super::json_escrita::{
    escrita_criar_mtr_json, escrita_json_mtr_criado, escrita_json_mtr_recebido,
    escrita_receber_mtr_json,
};

impl SinirAdapter {
    #[instrument(name = "sinir.criar_mtrs", skip_all)]
    /// Executa a operação SINIR `criar_mtrs`.
    pub(super) async fn criar_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        mtrs: Vec<CriarMtr>,
    ) -> Result<LoteResult<MtrCriado>, OpenMTRError> {
        let mut itens = Vec::with_capacity(mtrs.len());

        for (indice, criar) in mtrs.into_iter().enumerate() {
            let body = escrita_criar_mtr_json(&criar);
            match ctx
                .http
                .post::<Value, _>("/salvarManifestoLote", &body, "criar_mtr")
                .await
            {
                Ok(resp) => {
                    let erro_nacional = resp
                        .get("erroNacional")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    if erro_nacional {
                        let msg = resp
                            .get("mensagemErroNacional")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Erro nacional na criação");
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(msg),
                        });
                        continue;
                    }

                    let some_respostas = resp
                        .get("respostaApiwsManifestoDTO")
                        .and_then(Value::as_array)
                        .map_or_default(Vec::as_slice);

                    if some_respostas.is_empty() {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(
                                "Nenhuma resposta retornada pela API",
                            ),
                        });
                        continue;
                    }

                    for r in some_respostas {
                        let criado = escrita_json_mtr_criado(r);
                        if criado.sucesso {
                            itens.push(ItemResult::Ok(criado));
                        } else {
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::validation(criado.mensagem),
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!(erro = %e, "Falha ao criar MTR");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        info!(total = %itens.len(), "MTRs criados");
        Ok(LoteResult::new(itens))
    }
    #[instrument(name = "sinir.receber_mtrs", skip_all)]
    /// Executa a operação SINIR `receber_mtrs`.
    pub(super) async fn receber_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        lotes: Vec<ReceberMtr>,
    ) -> Result<LoteResult<MtrRecebido>, OpenMTRError> {
        let mut itens = Vec::with_capacity(lotes.len());

        for (indice, receber) in lotes.into_iter().enumerate() {
            let body = escrita_receber_mtr_json(&receber);
            match ctx
                .http
                .post::<Value, _>("/receberManifestoLote", &body, "receber_mtr")
                .await
            {
                Ok(resp) => {
                    let some_items = resp
                        .get("objeto")
                        .and_then(Value::as_array)
                        .map_or_default(Vec::as_slice);

                    if some_items.is_empty() {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation("Nenhum retorno no recebimento"),
                        });
                        continue;
                    }

                    for item in some_items {
                        let recebido = escrita_json_mtr_recebido(item);
                        if recebido.recebido {
                            itens.push(ItemResult::Ok(recebido));
                        } else {
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::validation(
                                    recebido.mensagem.unwrap_or_default(),
                                ),
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!(erro = %e, "Falha ao receber MTR");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        info!(total = %itens.len(), "MTRs recebidos");
        Ok(LoteResult::new(itens))
    }
}
