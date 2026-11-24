//! Downloads de MTR/CDF e emissão/consulta de CDFs no MTR-PE.

use futures::stream::{self, StreamExt};
use jiff::ToSpan;
use serde_json::{Value, json};
use tracing::instrument;

use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx, validate_download};
use crate::openmtr::domain::{Cdf, CdfEmitido, DownloadCdf, DownloadItem, FiltroCdf, GerarCdf};
use crate::openmtr::error::OpenMTRError;

use super::MtrPeAdapter;
use super::tipos::retorno_eh_sucesso;

impl MtrPeAdapter {
    #[instrument(name = "mtr_pe.download_mtrs", skip_all)]
    /// Baixa PDFs de MTRs por código de barras.
    pub(super) async fn download_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        nums: Vec<String>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        let tasks: Vec<_> = nums
            .into_iter()
            .enumerate()
            .map(|(indice, numero)| async move {
                let path = format!("/buscaPdfManifestoPorCodigoBarras/{numero}");
                match ctx.http.post_bytes(&path, "download_mtr").await {
                    Ok(response) => {
                        if let Some(problema) = validate_download(&response, "pdf") {
                            ItemResult::Erro { indice, problema }
                        } else {
                            ItemResult::Ok(DownloadItem {
                                filename: format!("{numero}.pdf"),
                                buffer: response.bytes,
                                formato: "pdf".into(),
                            })
                        }
                    }
                    Err(e) => ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    },
                }
            })
            .collect();

        let items: Vec<_> = stream::iter(tasks)
            .buffered(ctx.config.max_concurrency())
            .collect()
            .await;

        Ok(LoteResult::new(items))
    }

    #[instrument(name = "mtr_pe.gerar_cdfs", skip_all)]
    /// Emite CDFs via `/emiteCDF`.
    pub(super) async fn gerar_cdfs_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<GerarCdf>,
    ) -> Result<LoteResult<CdfEmitido>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let body = json!({
                "dataInicial": jiff::fmt::strtime::format("%Y%m%d", req.data_emissao).unwrap_or_default(),
                "dataFinal": jiff::fmt::strtime::format("%Y%m%d", req.data_emissao).unwrap_or_default(),
                "responsavel": req.responsavel,
                "txtObservacoes": req.observacoes,
                "listaGerador": [],
                "listaMtr": req.manifestos,
            });

            match ctx
                .http
                .post::<Value, _>("/emiteCDF", &body, "gerar_cdf")
                .await
            {
                Ok(resp) => {
                    if retorno_eh_sucesso(&resp) {
                        let cdf_obj = resp.get("listaCertificadoJSON");
                        if let Some(cdf) = cdf_obj {
                            if cdf.is_object() {
                                itens.push(ItemResult::Ok(CdfEmitido {
                                    codigo: cdf
                                        .get("codigo")
                                        .and_then(Value::as_i64)
                                        .map_or_default(|n| n.to_string()),
                                    numero: cdf
                                        .get("codigo")
                                        .and_then(Value::as_i64)
                                        .map_or_default(|n| n.to_string()),
                                    data_emissao: ctx.today(),
                                    status: String::new(),
                                }));
                            } else {
                                itens.push(ItemResult::Erro {
                                    indice,
                                    problema: OpenMTRError::validation("Resposta de CDF vazia"),
                                });
                            }
                        }
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

    #[instrument(name = "mtr_pe.consultar_cdfs", skip_all)]
    /// Consulta CDFs no período informado.
    pub(super) async fn consultar_cdfs_impl(
        &self,
        ctx: &MtrCtx,
        filtro: &FiltroCdf,
    ) -> Result<LoteResult<Cdf>, OpenMTRError> {
        let (data_ini, data_fim) = if let Some(p) = &filtro.periodo {
            (
                jiff::fmt::strtime::format("%d/%m/%Y", p.inicio.0).unwrap_or_default(),
                jiff::fmt::strtime::format("%d/%m/%Y", p.fim.0).unwrap_or_default(),
            )
        } else {
            let hoje = ctx.today();
            let mes_atras = hoje
                .checked_sub(ToSpan::days(30))
                .map_err(|e| OpenMTRError::validation(e.to_string()))?;
            (
                jiff::fmt::strtime::format("%d/%m/%Y", mes_atras).unwrap_or_default(),
                jiff::fmt::strtime::format("%d/%m/%Y", hoje).unwrap_or_default(),
            )
        };

        let body = json!({
            "cdfDataInicial": data_ini,
            "cdfDataFinal": data_fim,
            "cdfGeradorCodigo": null,
            "cdfDestinadorCodigo": null,
        });

        match ctx
            .http
            .post::<Value, _>("/consultaListaCdf", &body, "consultar_cdfs")
            .await
        {
            Ok(resp) => {
                let lista = resp
                    .get("listaCdfDTO")
                    .and_then(Value::as_array)
                    .map_or_default(Vec::as_slice);
                let cdfs: Vec<Cdf> = lista
                    .iter()
                    .map(|v| Cdf {
                        codigo: v
                            .get("cdfCodigo")
                            .and_then(|s| s.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        numero: v
                            .get("cdfCodigo")
                            .and_then(|s| s.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        data_emissao: ctx.today(),
                        status: String::new(),
                    })
                    .collect();
                Ok(LoteResult::from_vec(cdfs))
            }
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "mtr_pe.download_cdfs", skip_all)]
    /// Baixa PDFs de CDFs por código.
    pub(super) async fn download_cdfs_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<DownloadCdf>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        let tasks: Vec<_> = reqs
            .into_iter()
            .enumerate()
            .map(|(indice, req)| async move {
                let path = format!("/buscaPdfCdf/{}", req.codigo);
                match ctx.http.post_bytes(&path, "download_cdf").await {
                    Ok(response) => {
                        if let Some(problema) = validate_download(&response, "pdf") {
                            ItemResult::Erro { indice, problema }
                        } else {
                            ItemResult::Ok(DownloadItem {
                                filename: format!("{}.pdf", req.codigo),
                                buffer: response.bytes,
                                formato: "pdf".into(),
                            })
                        }
                    }
                    Err(e) => ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    },
                }
            })
            .collect();

        let items: Vec<_> = stream::iter(tasks)
            .buffered(ctx.config.max_concurrency())
            .collect()
            .await;

        Ok(LoteResult::new(items))
    }
}
