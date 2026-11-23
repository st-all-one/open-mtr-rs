//! Aceite ou recusa de alterações de recebimento no SIGOR.

use serde_json::{Value, json};
use tracing::instrument;

use super::SigorAdapter;
use super::tipos::tem_erro;
use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx};
use crate::openmtr::domain::{AceitarAlteracao, AlteracaoAceita};
use crate::openmtr::error::OpenMTRError;

impl SigorAdapter {
    /// Registra o aceite ou a recusa de alterações de recebimento em lote.
    #[instrument(name = "sigor.aceitar_alteracoes", skip_all)]
    pub(super) async fn aceitar_alteracoes_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<AceitarAlteracao>,
    ) -> Result<LoteResult<AlteracaoAceita>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let body = json!({
                "parCodigoGerador": req.codigo_gerador.unwrap_or(0),
                "manNumero": req.numero,
                "aceite": if req.aceitar { "a" } else { "r" },
            });

            match ctx
                .http
                .post::<Value, _>("/aceiteAlteracaoRecebimento", &body, "aceitar_alteracao")
                .await
            {
                Ok(resp) => {
                    let erro = tem_erro(&resp);
                    let msg = resp
                        .get("mensagem")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default();
                    if erro {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation(msg),
                        });
                    } else {
                        itens.push(ItemResult::Ok(AlteracaoAceita {
                            numero: req.numero,
                            aceita: req.aceitar,
                            mensagem: msg.to_string(),
                        }));
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
}
