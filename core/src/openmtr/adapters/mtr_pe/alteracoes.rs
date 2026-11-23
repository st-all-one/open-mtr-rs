//! Aceite de alterações de recebimento no MTR-PE.

use serde_json::{Value, json};
use tracing::{instrument, warn};

use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx};
use crate::openmtr::domain::{AceitarAlteracao, AlteracaoAceita};
use crate::openmtr::error::OpenMTRError;

use super::MtrPeAdapter;

impl MtrPeAdapter {
    #[instrument(name = "mtr_pe.aceitar_alteracoes", skip_all)]
    /// Aceita ou rejeita alterações de recebimento.
    pub(super) async fn aceitar_alteracoes_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<AceitarAlteracao>,
    ) -> Result<LoteResult<AlteracaoAceita>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let num = req.numero.parse::<i64>().unwrap_or(0);
            let body = json!({
                "codigoManifesto": num,
                "aceite": if req.aceitar { "S" } else { "N" },
            });

            match ctx
                .http
                .post::<Value, _>("/aceiteAlteracaoRecebimento", &body, "aceitar_alteracao")
                .await
            {
                Ok(resp) => {
                    let retorno_codigo = resp
                        .get("retornoCodigo")
                        .and_then(Value::as_i64)
                        .unwrap_or(999);
                    let msg = resp
                        .get("retorno")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default()
                        .to_string();
                    if retorno_codigo == 0 {
                        itens.push(ItemResult::Ok(AlteracaoAceita {
                            numero: req.numero,
                            aceita: req.aceitar,
                            mensagem: msg,
                        }));
                    } else {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(msg),
                        });
                    }
                }
                Err(e) => {
                    warn!(erro = %e, "Falha ao aceitar alteração no MTR-PE");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        Ok(LoteResult::new(itens))
    }
}
