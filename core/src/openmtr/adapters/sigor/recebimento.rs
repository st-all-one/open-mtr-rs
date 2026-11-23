//! Recebimento e modificação de manifestos no SIGOR.

use serde_json::{Value, json};
use tracing::instrument;

use super::SigorAdapter;
use super::tipos::json_para_mtr_recebido;
use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx};
use crate::openmtr::domain::{
    ModificacaoTipo, ModificarMtr, MtrModificado, MtrRecebido, ReceberMtr,
};
use crate::openmtr::error::OpenMTRError;

impl SigorAdapter {
    /// Recebe manifestos em lote via `/receberManifestoLote`.
    #[instrument(name = "sigor.receber_mtrs", skip_all)]
    pub(super) async fn receber_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        lotes: Vec<ReceberMtr>,
    ) -> Result<LoteResult<MtrRecebido>, OpenMTRError> {
        let mut itens = Vec::with_capacity(lotes.len());

        for (indice, receber) in lotes.into_iter().enumerate() {
            let res_items: Vec<Value> = receber
                .residuos_recebidos
                .iter()
                .map(|r| {
                    let mut item = json!({
                        "resCodigoIbama": r.codigo_ibama,
                        "marQuantidade": r.quantidade_recebida,
                        "marQuantidadeRecebida": r.quantidade_recebida,
                        "traCodigo": r.tratamento,
                        "uniCodigo": r.unidade,
                    });
                    if let Some(ref n) = r.codigo_ibama_novo {
                        item["resCodigoIbamaNovo"] = json!(n);
                    }
                    if let Some(v) = r.tratamento_novo {
                        item["traCodigoNovo"] = json!(v);
                    }
                    if let Some(v) = r.classe_nova {
                        item["claCodigoNovo"] = json!(v);
                    }
                    if let Some(v) = r.estado_fisico {
                        item["tieCodigo"] = json!(v);
                    }
                    if let Some(v) = r.acondicionamento {
                        item["tiaCodigo"] = json!(v);
                    }
                    if let Some(v) = r.classe {
                        item["claCodigo"] = json!(v);
                    }
                    item
                })
                .collect();

            let body = json!([{
                "manNumero": receber.numero,
                "dataRecebimento": receber.data_recebimento.and_utc().timestamp_millis(),
                "nomeMotorista": receber.motorista.as_deref().unwrap_or("Motorista"),
                "placaVeiculo": receber.placa_veiculo.as_deref().unwrap_or("ABC1234"),
                "nomeResponsavelRecebimento": receber.responsavel,
                "observacoes": receber.observacoes,
                "listaManifestoResiduo": res_items,
            }]);

            match ctx
                .http
                .post::<Value, _>("/receberManifestoLote", &body, "receber_mtr")
                .await
            {
                Ok(resp) => {
                    let dtos = resp
                        .get("objetoResposta")
                        .and_then(Value::as_array)
                        .map_or_default(Vec::as_slice);
                    if dtos.is_empty() {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation("Resposta vazia"),
                        });
                        continue;
                    }
                    for dto in dtos {
                        let recebido = json_para_mtr_recebido(dto);
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
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }

    /// Modifica o recebimento de manifestos no SIGOR.
    #[instrument(name = "sigor.modificar_mtrs", skip_all)]
    pub(super) async fn modificar_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<ModificarMtr>,
    ) -> Result<LoteResult<MtrModificado>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            match req.tipo {
                ModificacaoTipo::Recebimento => {
                    let novos = req.novos_residuos.unwrap_or_default();
                    let novos_itens: Vec<Value> = novos
                        .iter()
                        .map(|r| {
                            let mut item = json!({
                                "resCodigoIbama": r.codigo_ibama,
                                "marQuantidade": r.quantidade_recebida,
                                "marQuantidadeRecebida": r.quantidade_recebida,
                                "traCodigo": r.tratamento,
                                "claCodigo": r.classe,
                            });
                            if let Some(v) = r.classe_nova {
                                item["claCodigoNovo"] = json!(v);
                            }
                            if let Some(v) = r.tratamento_novo {
                                item["traCodigoNovo"] = json!(v);
                            }
                            item
                        })
                        .collect();

                    let body = json!({
                            "parCodigoDestinador": 0,
                            "remObservacao": req.justificativa,
                            "manifesto": {
                                "manNumero": req.numero,
                    "listaManifestoResiduo": novos_itens,
                            }
                        });

                    match ctx
                        .http
                        .post::<Value, _>("/alteraRecebimento", &body, "modificar_mtr")
                        .await
                    {
                        Ok(_) => {
                            itens.push(ItemResult::Ok(MtrModificado {
                                numero: req.numero,
                                modificado: true,
                                mensagem: "Solicitação de alteração enviada".into(),
                            }));
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
                            "Tipo de modificação não suportado no SIGOR: {:?}",
                            req.tipo
                        )),
                    });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }
}
