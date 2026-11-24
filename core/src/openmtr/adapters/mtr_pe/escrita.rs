//! Escrita de manifestos no MTR-PE: criação, recebimento, cancelamento e alteração.

use serde_json::{Value, json};
use tracing::{instrument, warn};

use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx};
use crate::openmtr::domain::{
    CancelarMtr, CriarMtr, ModificacaoTipo, ModificarMtr, MtrCancelado, MtrCriado, MtrModificado,
    MtrRecebido, ReceberMtr,
};
use crate::openmtr::error::OpenMTRError;

use super::MtrPeAdapter;
use super::tipos::{criar_mtr_para_json, json_para_mtr_criado, retorno_eh_sucesso};

impl MtrPeAdapter {
    #[instrument(name = "mtr_pe.criar_mtrs", skip_all)]
    /// Cria MTRs em lote via `/salvarManifestoLote`.
    pub(super) async fn criar_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        mtrs: Vec<CriarMtr>,
    ) -> Result<LoteResult<MtrCriado>, OpenMTRError> {
        let mut itens = Vec::with_capacity(mtrs.len());

        for (indice, criar) in mtrs.into_iter().enumerate() {
            let body = json!({
                "manifestoJSONDtos": [criar_mtr_para_json(&criar)]
            });

            match ctx
                .http
                .post::<Value, _>("/salvarManifestoLote", &body, "criar_mtr")
                .await
            {
                Ok(resp) => {
                    let dtos = resp
                        .get("manifestoJSONDtos")
                        .and_then(Value::as_array)
                        .map_or_default(Vec::as_slice);
                    if dtos.is_empty() {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation("Resposta vazia do MTR-PE"),
                        });
                        continue;
                    }
                    for dto in dtos {
                        let criado = json_para_mtr_criado(dto);
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
                    warn!(erro = %e, "Falha ao criar MTR no MTR-PE");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }

    #[instrument(name = "mtr_pe.receber_mtrs", skip_all)]
    /// Recebe MTRs em lote via `/receberManifestoLote`.
    pub(super) async fn receber_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        lotes: Vec<ReceberMtr>,
    ) -> Result<LoteResult<MtrRecebido>, OpenMTRError> {
        let mut itens = Vec::with_capacity(lotes.len());

        for (indice, receber) in lotes.into_iter().enumerate() {
            let body: Vec<Value> = receber
                .residuos_recebidos
                .iter()
                .map(|r| {
                    json!({
                        "codigoSequencial": Option::<i32>::None,
                        "qtdRecebida": r.quantidade_recebida,
                        "residuo": r.codigo_ibama,
                        "codigoTecnologia": r.tratamento,
                    })
                })
                .collect();

            let body = json!({
                "manifestoRecebimentoJSONs": [{
                    "manifestoCodigo": receber.numero,
                    "recebimentoMtrResponsavel": receber.responsavel,
                    "recebimentoMtrCargo": "Responsavel",
                    "recebimentoMtrData": jiff::fmt::strtime::format("%Y%m%d", receber.data_recebimento).unwrap_or_default(),
                    "transporteMtrData": jiff::fmt::strtime::format("%Y%m%d", receber.data_recebimento).unwrap_or_default(),
                    "itemManifestoRecebimentoJSONs": body,
                }]
            });

            match ctx
                .http
                .post::<Value, _>("/receberManifestoLote", &body, "receber_mtr")
                .await
            {
                Ok(resp) => {
                    let dtos = resp
                        .get("manifestoRecebimentoJSONs")
                        .and_then(Value::as_array)
                        .map_or_default(Vec::as_slice);
                    for dto in dtos {
                        if retorno_eh_sucesso(dto) {
                            itens.push(ItemResult::Ok(MtrRecebido {
                                numero: receber.numero.clone(),
                                recebido: true,
                                mensagem: dto
                                    .get("retorno")
                                    .and_then(|s| s.as_str())
                                    .map(String::from),
                            }));
                        } else {
                            let msg = dto
                                .get("retorno")
                                .and_then(|s| s.as_str())
                                .unwrap_or("Erro");
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::validation(msg),
                            });
                        }
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

        Ok(LoteResult::new(itens))
    }

    #[instrument(name = "mtr_pe.cancelar_mtrs", skip_all)]
    /// Cancela MTRs via `/cancelarManifesto`.
    pub(super) async fn cancelar_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<CancelarMtr>,
    ) -> Result<LoteResult<MtrCancelado>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let body = json!({
                "manifestoCodigo": req.numero,
                "justificativa": req.justificativa,
            });

            match ctx
                .http
                .post::<Value, _>("/cancelarManifesto", &body, "cancelar_mtr")
                .await
            {
                Ok(resp) => {
                    if retorno_eh_sucesso(&resp) {
                        itens.push(ItemResult::Ok(MtrCancelado {
                            numero: resp
                                .get("manifestoCodigo")
                                .and_then(Value::as_i64)
                                .map_or_default(|n| n.to_string()),
                            status: "Cancelado".into(),
                            data_cancelamento: None,
                        }));
                    } else {
                        let msg = resp
                            .get("retorno")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Erro");
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(msg),
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

        Ok(LoteResult::new(itens))
    }

    #[instrument(name = "mtr_pe.modificar_mtrs", skip_all)]
    /// Modifica MTRs recebidos via `/alteraRecebimento`.
    pub(super) async fn modificar_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<ModificarMtr>,
    ) -> Result<LoteResult<MtrModificado>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            match req.tipo {
                ModificacaoTipo::Recebimento => {
                    let body = json!({
                        "manifestoAlterarJSONs": [{
                            "manifestoCodigo": req.numero.parse::<i64>().unwrap_or(0),
                            "residuo": "",
                            "qtdRecebidaNovo": req.novos_residuos.as_ref().and_then(|r| r.first().map(|x| x.quantidade_recebida)),
                            "codigoClasseNovo": req.novos_residuos.as_ref().and_then(|r| r.first().and_then(|x| x.classe)),
                            "codigoTecnologiaNovo": req.novos_residuos.as_ref().and_then(|r| r.first().and_then(|x| x.tratamento)),
                            "codigoUnidadeNovo": req.novos_residuos.as_ref().and_then(|r| r.first().map(|x| x.unidade)),
                        }]
                    });

                    match ctx
                        .http
                        .post::<Value, _>("/alteraRecebimento", &body, "modificar_mtr")
                        .await
                    {
                        Ok(resp) => {
                            let dtos = resp
                                .get("manifestoAlterarJSONs")
                                .and_then(Value::as_array)
                                .map_or_default(Vec::as_slice);
                            if let Some(dto) = dtos.first() {
                                if retorno_eh_sucesso(dto) {
                                    itens.push(ItemResult::Ok(MtrModificado {
                                        numero: req.numero,
                                        modificado: true,
                                        mensagem: dto
                                            .get("retorno")
                                            .and_then(|s| s.as_str())
                                            .unwrap_or("OK")
                                            .to_string(),
                                    }));
                                } else {
                                    let msg = dto
                                        .get("retorno")
                                        .and_then(|s| s.as_str())
                                        .unwrap_or("Erro");
                                    itens.push(ItemResult::Erro {
                                        indice,
                                        problema: OpenMTRError::validation(msg),
                                    });
                                }
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
                _ => {
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::validation(format!(
                            "Tipo de modificação não suportado no MTR-PE: {:?}",
                            req.tipo
                        )),
                    });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }
}
