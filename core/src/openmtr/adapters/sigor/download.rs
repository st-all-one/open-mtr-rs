//! Download de manifestos e certificados do SIGOR.

use futures::stream::{self, StreamExt};
use tracing::instrument;

use super::SigorAdapter;
use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx, validate_download};
use crate::openmtr::domain::{DownloadCdf, DownloadItem};
use crate::openmtr::error::OpenMTRError;

impl SigorAdapter {
    /// Baixa PDFs de manifestos com concorrência limitada.
    #[instrument(name = "sigor.download_mtrs", skip_all)]
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

    /// Baixa PDFs de certificados com concorrência limitada.
    #[instrument(name = "sigor.download_cdfs", skip_all)]
    pub(super) async fn download_cdfs_impl(
        &self,
        ctx: &MtrCtx,
        reqs: Vec<DownloadCdf>,
    ) -> Result<LoteResult<DownloadItem>, OpenMTRError> {
        let tasks: Vec<_> = reqs
            .into_iter()
            .enumerate()
            .map(|(indice, req)| async move {
                let path = format!("/downloadCertificado/{}", req.codigo);
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
