//! Operações SINIR: download.

use futures::stream::{self, StreamExt};
use serde_json::{Value, json};
use tracing::{info, instrument, warn};

use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx, validate_download};
use crate::openmtr::domain::{
    Cdf, CdfEmitido, DownloadCdf, DownloadItem, FiltroCdf, FormatoDownload, GerarCdf,
};
use crate::openmtr::error::OpenMTRError;

use super::SinirAdapter;
use super::json_listas::lista_cdf_emitido;

impl SinirAdapter {
    #[instrument(name = "sinir.download_mtrs", skip_all)]
    /// Executa a operação SINIR `download_mtrs`.
    pub(super) async fn download_mtrs_impl(
        &self,
        ctx: &MtrCtx,
        nums: Vec<String>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        let tasks: Vec<_> = nums
            .into_iter()
            .enumerate()
            .map(|(indice, numero)| async move {
                let path = format!("/downloadManifesto/{numero}");
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
                    Err(e) => {
                        warn!(numero = %numero, erro = %e, "Falha ao baixar MTR");
                        ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::api_error(502, e.to_string()),
                        }
                    }
                }
            })
            .collect();

        let items: Vec<_> = stream::iter(tasks)
            .buffered(ctx.config.max_concurrency())
            .collect()
            .await;

        info!(total = %items.len(), "MTRs baixados");
        Ok(LoteResult::new(items))
    }
    #[instrument(name = "sinir.gerar_cdfs", skip_all)]
    /// Executa a operação SINIR `gerar_cdfs`.
    pub(super) async fn gerar_cdfs_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<GerarCdf>,
    ) -> Result<LoteResult<CdfEmitido>, OpenMTRError> {
        let mut itens = Vec::with_capacity(reqs.len());

        for (indice, req) in reqs.into_iter().enumerate() {
            let body = json!({
                "listaManifestos": req.manifestos,
                "dataEmissao": req.data_emissao.at(0, 0, 0, 0)
                    .to_zoned_datetime(jiff::tz::TimeZone::UTC)
                    .map_or(0, |zdt| zdt.timestamp().as_millisecond()),
                "nomeResponsavel": req.responsavel,
                "observacoes": req.observacoes,
            });

            match ctx
                .http
                .post::<Value, _>("/emiteCDF", &body, "gerar_cdf")
                .await
            {
                Ok(resp) => match resp.get("objeto") {
                    Some(objeto) => {
                        itens.push(ItemResult::Ok(lista_cdf_emitido(objeto, ctx.today())));
                    }
                    None => {
                        itens.push(ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::validation("Resposta vazia na emissão de CDF"),
                        });
                    }
                },
                Err(e) => {
                    warn!(erro = %e, "Falha ao emitir CDF");
                    itens.push(ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    });
                }
            }
        }

        info!(total = %itens.len(), "CDFs emitidos");
        Ok(LoteResult::new(itens))
    }
    /// Executa a operação SINIR `consultar_cdfs`.
    pub(super) fn consultar_cdfs_impl(
        _ctx: &MtrCtx,
        _filtro: &FiltroCdf,
    ) -> Result<LoteResult<Cdf>, OpenMTRError> {
        Err(OpenMTRError::not_supported(
            "SinirAdapter::consultar_cdfs — sem endpoint dedicado no SINIR",
        ))
    }
    /// Executa a operação SINIR `download_cdfs`.
    #[instrument(name = "sinir.download_cdfs", skip_all)]
    pub(super) async fn download_cdfs_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<DownloadCdf>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        let tasks: Vec<_> = reqs
            .into_iter()
            .enumerate()
            .map(|(indice, req)| async move {
                let formato_str = match req.formato {
                    FormatoDownload::Pdf => "pdf",
                    FormatoDownload::Xml => "xml",
                };
                let path = format!("/downloadCertificado/{}?formato={formato_str}", req.codigo);
                match ctx.http.post_bytes(&path, "download_cdf").await {
                    Ok(response) => {
                        if let Some(problema) = validate_download(&response, formato_str) {
                            ItemResult::Erro { indice, problema }
                        } else {
                            ItemResult::Ok(DownloadItem {
                                filename: format!("{}.{formato_str}", req.codigo),
                                buffer: response.bytes,
                                formato: formato_str.into(),
                            })
                        }
                    }
                    Err(e) => {
                        warn!(codigo = %req.codigo, erro = %e, "Falha ao baixar CDF");
                        ItemResult::Erro {
                            indice,
                            problema: OpenMTRError::api_error(502, e.to_string()),
                        }
                    }
                }
            })
            .collect();

        let items: Vec<_> = stream::iter(tasks)
            .buffered(ctx.config.max_concurrency())
            .collect()
            .await;

        info!(total = %items.len(), "CDFs baixados");
        Ok(LoteResult::new(items))
    }
}
