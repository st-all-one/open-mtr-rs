//! Emissão e consulta de CDFs no SIGOR.

use serde_json::{Value, json};
use tracing::instrument;

use super::SigorAdapter;
use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx};
use crate::openmtr::domain::{Cdf, CdfEmitido, FiltroCdf, GerarCdf};
use crate::openmtr::error::OpenMTRError;

impl SigorAdapter {
    /// Emite CDFs em lote via `/emiteCDF`.
    #[instrument(name = "sigor.gerar_cdfs", skip_all)]
    pub(super) async fn gerar_cdfs_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<GerarCdf>,
    ) -> Result<LoteResult<CdfEmitido>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let data_str =
                jiff::fmt::strtime::format("%Y%m%d", req.data_emissao).unwrap_or_default();
            let body = json!({
                "cerDataInicial": data_str,
                "cerDataFinal": data_str,
                "parceiroDestinador": "",
                "cnpjDestinador": "",
                "tipoCertificadoDestinacao": 1,
                "listaManifesto": req.manifestos,
                "listaParceiroGerador": [],
                "nomeResponsavel": req.responsavel,
                "cerObservacao": req.observacoes,
            });

            match ctx
                .http
                .post::<Value, _>("/emiteCDF", &body, "gerar_cdf")
                .await
            {
                Ok(resp) => {
                    let valido = resp
                        .get("restResponseValido")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    if valido {
                        itens.push(ItemResult::Ok(CdfEmitido {
                            codigo: resp
                                .get("numeroCDF")
                                .and_then(Value::as_i64)
                                .map_or_default(|n| n.to_string()),
                            numero: String::new(),
                            data_emissao: ctx.today(),
                            status: String::new(),
                        }));
                    } else {
                        let msg = resp
                            .get("restResponseMensagem")
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

    /// Consulta de CDFs não suportada pelo SIGOR (sem endpoint dedicado).
    pub(super) fn consultar_cdfs_impl(
        _ctx: &MtrCtx,
        _filtro: &FiltroCdf,
    ) -> Result<LoteResult<Cdf>, OpenMTRError> {
        Err(OpenMTRError::not_supported(
            "SIGOR: consultar_cdfs sem endpoint dedicado",
        ))
    }
}
