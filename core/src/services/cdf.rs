use crate::client::http::MtrClient;
use crate::core::errors::{ErrorCategory, ErrorContext, MtrError};
use crate::schemas::cdf::{
    validar_emissao, ApiwsCertificadoDestinacaoDTO, CDFEmissaoObjeto, FormatoDownload,
};
use crate::schemas::common::ObjetoRespostaRest;

#[derive(Clone)]
pub struct CdfService {
    client: MtrClient,
}

impl CdfService {
    #[must_use]
    pub fn new(client: MtrClient) -> Self {
        Self { client }
    }

    pub async fn emitir(
        &self,
        payload: &ApiwsCertificadoDestinacaoDTO,
    ) -> Result<CDFEmissaoObjeto, MtrError> {
        validar_emissao(payload)?;

        let raw: ObjetoRespostaRest<CDFEmissaoObjeto> = self
            .client
            .request_with_body(reqwest::Method::POST, "/emiteCDF", payload, "cdf.emitir")
            .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Emissão CDF falhou",
                ErrorContext::new("cdf.emitir"),
            )
        })
    }

    pub async fn emitir_lr(
        &self,
        payload: &ApiwsCertificadoDestinacaoDTO,
    ) -> Result<CDFEmissaoObjeto, MtrError> {
        validar_emissao(payload)?;

        let raw: ObjetoRespostaRest<CDFEmissaoObjeto> = self
            .client
            .request_with_body(reqwest::Method::POST, "/emiteCDFLR", payload, "cdf.emitir_lr")
            .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Emissão CDF LR falhou",
                ErrorContext::new("cdf.emitir_lr"),
            )
        })
    }

    pub async fn download(
        &self,
        cdf_codigo: &str,
        formato: FormatoDownload,
    ) -> Result<Vec<u8>, MtrError> {
        self.client
            .download_raw(
                reqwest::Method::POST,
                &format!(
                    "/downloadCertificado/{cdf_codigo}?formato={}",
                    formato
                ),
            )
            .await
    }

    pub async fn salvar(
        &self,
        cdf_codigo: &str,
        formato: FormatoDownload,
        caminho: impl AsRef<std::path::Path>,
    ) -> Result<(), MtrError> {
        let bytes = self.download(cdf_codigo, formato).await?;
        tokio::fs::write(&caminho, &bytes)
            .await
            .map_err(|e| MtrError::config(format!("Falha ao salvar CDF: {e}")))
    }
}
