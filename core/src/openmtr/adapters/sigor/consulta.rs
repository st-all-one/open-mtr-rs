//! Consulta e criação de manifestos no SIGOR.

use serde_json::{Value, json};
use tracing::{instrument, warn};

use super::SigorAdapter;
use super::tipos::{
    criar_mtr_para_json, extrair_objeto, json_para_mtr, json_para_mtr_criado, tem_erro,
};
use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx};
use crate::openmtr::domain::{CriarMtr, FiltroMtr, Mtr, MtrCriado};
use crate::openmtr::error::OpenMTRError;

impl SigorAdapter {
    /// Consulta manifestos pelo número; não suporta consulta por período.
    #[instrument(name = "sigor.consultar_mtrs", skip_all)]
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
                match ctx.http.get::<Value>(&path, "consultar_mtr").await {
                    Ok(resp) => {
                        if tem_erro(&resp) {
                            let msg = resp
                                .get("mensagem")
                                .and_then(|s| s.as_str())
                                .unwrap_or("Erro");
                            itens.push(ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::not_found(msg),
                            });
                        } else {
                            match extrair_objeto(&resp) {
                                Some(detail) => itens.push(ItemResult::Ok(json_para_mtr(detail))),
                                None => itens.push(ItemResult::Erro {
                                    indice,
                                    problema: OpenMTRError::not_found("Resposta vazia do SIGOR"),
                                }),
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
            return Ok(LoteResult::new(itens));
        }

        Err(OpenMTRError::not_supported(
            "SIGOR: consulta sem número via lista de período não implementada",
        ))
    }

    /// Cria manifestos em lote via `/salvarManifestoLote`.
    #[instrument(name = "sigor.criar_mtrs", skip_all)]
    pub(super) async fn criar_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        mtrs: Vec<CriarMtr>,
    ) -> Result<LoteResult<MtrCriado>, OpenMTRError> {
        let mut itens = Vec::with_capacity(mtrs.len());

        for (indice, criar) in mtrs.into_iter().enumerate() {
            let body = criar_mtr_para_json(&criar);

            match ctx
                .http
                .post::<Value, _>("/salvarManifestoLote", &body, "criar_mtr")
                .await
            {
                Ok(resp) => {
                    if resp
                        .get("restResponseValido")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                    {
                        itens.push(ItemResult::Ok(json_para_mtr_criado(&resp)));
                    } else {
                        let msg = resp
                            .get("restResponseMensagem")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Erro na criação");
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(msg),
                        });
                    }
                }
                Err(e) => {
                    warn!(erro = %e, "Falha ao criar MTR no SIGOR");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }

    /// Consulta um manifesto pelo código de referência da unidade.
    #[instrument(name = "sigor.consultar_mtr_por_codigo_referencia", skip_all)]
    pub(super) async fn consultar_mtr_por_codigo_referencia_impl(
        &self,
        ctx: &MtrCtx,
        unidade: u32,
        codigo: &str,
    ) -> Result<Option<Mtr>, OpenMTRError> {
        let body = json!({ "suaUnidade": unidade, "seuCodigo": codigo });
        let response: Value = ctx
            .http
            .post("/retornaManifestoSeuCodigo", &body, "consultar_por_codigo")
            .await?;
        if tem_erro(&response) {
            return Ok(None);
        }
        match extrair_objeto(&response) {
            Some(detail) => Ok(Some(json_para_mtr(detail))),
            None => Ok(None),
        }
    }
}
