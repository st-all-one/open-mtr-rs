//! Consulta de MTRs por período via códigos de barras no MTR-PE.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::stream::{self, StreamExt};
use jiff::ToSpan;
use serde_json::{Value, json};
use tracing::{info, warn};

use crate::openmtr::adapters::{ItemResult, LoteResult, MtrCtx};
use crate::openmtr::domain::{Mtr, Periodo};
use crate::openmtr::error::OpenMTRError;

use super::MtrPeAdapter;
use super::tipos::{json_para_mtr, manifesto_recuperavel, retorno_eh_sucesso};

impl MtrPeAdapter {
    /// Lista MTRs por período no MTR-PE.
    ///
    /// A API estadual não tem listagem estruturada — `retornaListaCodigoBarasManifesto`
    /// devolve apenas códigos de barras (janela máxima de 30 dias). Este método
    /// percorre o período em janelas de 30 dias, deduplica os barcodes e consulta
    /// o detalhe de cada manifesto em paralelo (concorrência limitada).
    pub(super) async fn consultar_mtrs_por_periodo_impl(
        &self,
        ctx: &MtrCtx,
        periodo: &Periodo,
    ) -> Result<LoteResult<Mtr>, OpenMTRError> {
        const JANELA_DIAS: i64 = 30;
        const CONCORRENCIA: usize = 10;

        let inicio = periodo.inicio.0;
        let fim = periodo.fim.0;

        // 1. Coleta barcodes em janelas de 30 dias
        let mut barcodes: Vec<String> = Vec::new();
        let mut atual = inicio;
        while atual <= fim {
            let Some(janela_fim) = atual
                .checked_add(ToSpan::days(JANELA_DIAS - 1))
                .ok()
                .map(|d| d.min(fim))
            else {
                break;
            };
            let path = format!(
                "/retornaListaCodigoBarasManifesto/{}/{}",
                jiff::fmt::strtime::format("%Y%m%d", atual).unwrap_or_default(),
                jiff::fmt::strtime::format("%Y%m%d", janela_fim).unwrap_or_default()
            );
            let resp: Value = ctx.http.post(&path, &json!({}), "listar_barcodes").await?;
            if let Some(codigos) = resp.get("codigos").and_then(Value::as_array) {
                barcodes.extend(codigos.iter().filter_map(|c| c.as_str().map(String::from)));
            }
            let Some(proxima) = janela_fim.checked_add(ToSpan::days(1)).ok() else {
                break;
            };
            atual = proxima;
        }

        barcodes.sort();
        barcodes.dedup();

        if barcodes.is_empty() {
            info!(
                inicio = %inicio,
                fim = %fim,
                "Nenhum código de barras no período"
            );
            return Ok(LoteResult::new(Vec::new()));
        }

        let total = i64::try_from(barcodes.len()).unwrap_or(i64::MAX);

        // 2. Busca o detalhe de cada barcode (concorrência limitada)
        let parciais = Arc::new(AtomicUsize::new(0));
        let tasks = barcodes.into_iter().enumerate().map(|(indice, barcode)| {
            let parciais = Arc::clone(&parciais);
            async move {
                let path = format!("/retornaManifesto/{barcode}");
                match ctx
                    .http
                    .post::<Value, _>(&path, &json!({}), "consultar_mtr")
                    .await
                {
                    Ok(resp) => {
                        if retorno_eh_sucesso(&resp) {
                            ItemResult::Ok(json_para_mtr(&resp, &barcode))
                        } else if manifesto_recuperavel(&resp) {
                            parciais.fetch_add(1, Ordering::Relaxed);
                            ItemResult::Ok(json_para_mtr(&resp, &barcode))
                        } else {
                            let msg = resp
                                .get("retorno")
                                .and_then(|s| s.as_str())
                                .unwrap_or("Erro");
                            ItemResult::Erro {
                                indice,
                                problema: OpenMTRError::not_found(msg),
                            }
                        }
                    }
                    Err(e) => ItemResult::Erro {
                        indice,
                        problema: OpenMTRError::api_error(502, e.to_string()),
                    },
                }
            }
        });

        let itens: Vec<_> = stream::iter(tasks).buffered(CONCORRENCIA).collect().await;
        let parciais = parciais.load(Ordering::Relaxed);

        info!(
            total = %total,
            sucessos = %itens.iter().filter(|i| matches!(i, ItemResult::Ok(_))).count(),
            parciais = %parciais,
            "MTRs consultados por período via códigos de barras"
        );
        if parciais > 0 {
            warn!(
                parciais,
                total = %total,
                "MTR-PE: manifesto sem CDF; emitido só com o cabeçalho"
            );
        }

        let mut lote = LoteResult::new(itens);
        lote.total = Some(total);
        Ok(lote)
    }
}
