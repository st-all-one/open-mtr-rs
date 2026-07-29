use crate::client::http::MtrClient;
use crate::core::errors::{ErrorCategory, ErrorContext, MtrError};
use crate::schemas::common::ObjetoRespostaRest;
use crate::schemas::integracao_estadual::{ApiwsManifestoDTO, RespostaIntegracaoObjeto};

#[derive(Clone)]
pub struct IntegracaoEstadualService {
    client: MtrClient,
}

impl IntegracaoEstadualService {
    #[must_use]
    pub fn new(client: MtrClient) -> Self {
        Self { client }
    }

    pub async fn integrar_manifestos(
        &self,
        manifestos: &[ApiwsManifestoDTO],
    ) -> Result<RespostaIntegracaoObjeto, MtrError> {
        if manifestos.is_empty() {
            return Err(MtrError::validation("Lista de manifestos vazia"));
        }

        for m in manifestos {
            m.gerador.validar()?;
            m.transportador.validar()?;
            m.destinador.validar()?;
        }

        let raw: ObjetoRespostaRest<RespostaIntegracaoObjeto> = self
            .client
            .request_with_body(
                reqwest::Method::POST,
                "/integraMTREstadual",
                manifestos,
                "integracao.integrar_manifestos",
            )
            .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Integração falhou",
                ErrorContext::new("integracao.integrar_manifestos"),
            )
        })
    }

    pub async fn integrar_recebimento(
        &self,
        recebimento: &serde_json::Value,
    ) -> Result<serde_json::Value, MtrError> {
        let raw: ObjetoRespostaRest<serde_json::Value> = self
            .client
            .request_with_body(
                reqwest::Method::POST,
                "/integraRecebimentoMTREstadual",
                recebimento,
                "integracao.integrar_recebimento",
            )
            .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Integração de recebimento falhou",
                ErrorContext::new("integracao.integrar_recebimento"),
            )
        })
    }
}

pub struct LoteResult {
    pub processados: i32,
    pub falhas: Vec<FalhaLote>,
}

pub struct FalhaLote {
    pub man_numero: String,
    pub erro: String,
}

pub async fn integrar_lote_inteligente(
    service: &IntegracaoEstadualService,
    manifestos: Vec<ApiwsManifestoDTO>,
) -> LoteResult {
    let mut falhas = Vec::new();
    let mut processados = 0i32;

    for chunk in manifestos.chunks(50) {
        match service.integrar_manifestos(chunk).await {
            Ok(resp) => {
                processados += resp.manifestos_processados;
                for erro in resp.erros {
                    falhas.push(FalhaLote {
                        man_numero: erro.man_numero,
                        erro: erro.mensagem,
                    });
                }
            }
            Err(_) => {
                for m in chunk {
                    match service.integrar_manifestos(&[m.clone()]).await {
                        Ok(r) => processados += r.manifestos_processados,
                        Err(e2) => {
                            falhas.push(FalhaLote {
                                man_numero: m.man_numero.clone(),
                                erro: e2.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    LoteResult { processados, falhas }
}
