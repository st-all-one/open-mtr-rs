use crate::client::http::MtrClient;
use crate::core::errors::{ErrorCategory, ErrorContext, MtrError};
use crate::schemas::common::{DataMtr, ObjetoRespostaRest};
use crate::schemas::logistica_reversa::{
    ApiwsAlteraManifestoOrigemDTO, MtrLrDataDTO, MtrOrigemCompletoDTO, MtrOrigemCompletoObjeto,
    MtrProcessoCompletoDTO,
};

#[derive(Clone)]
pub struct LogisticaReversaService {
    client: MtrClient,
}

impl LogisticaReversaService {
    #[must_use]
    pub fn new(client: MtrClient) -> Self {
        Self { client }
    }

    pub async fn listar_origem(
        &self,
        data_inicial: DataMtr,
        data_final: DataMtr,
    ) -> Result<serde_json::Value, MtrError> {
        validar_periodo(&data_inicial, &data_final)?;

        let filtro = MtrLrDataDTO {
            data_inicial,
            data_final,
        };

        let raw: ObjetoRespostaRest<serde_json::Value> = self
            .client
            .request_with_body(
                reqwest::Method::POST,
                "/listaMtrOrigem",
                &filtro,
                "lr.listar_origem",
            )
            .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Lista origem vazia",
                ErrorContext::new("lr.listar_origem"),
            )
        })
    }

    pub async fn listar_processo(
        &self,
        data_inicial: DataMtr,
        data_final: DataMtr,
    ) -> Result<serde_json::Value, MtrError> {
        validar_periodo(&data_inicial, &data_final)?;

        let filtro = MtrLrDataDTO {
            data_inicial,
            data_final,
        };

        let raw: ObjetoRespostaRest<serde_json::Value> = self
            .client
            .request_with_body(
                reqwest::Method::POST,
                "/listaMtrProcesso",
                &filtro,
                "lr.listar_processo",
            )
            .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Lista processo vazia",
                ErrorContext::new("lr.listar_processo"),
            )
        })
    }

    pub async fn obter_origem_completo(
        &self,
        man_numero: &str,
    ) -> Result<MtrOrigemCompletoObjeto, MtrError> {
        let payload = MtrOrigemCompletoDTO {
            man_numero: man_numero.to_string(),
        };

        let raw: ObjetoRespostaRest<MtrOrigemCompletoObjeto> = self
            .client
            .request_with_body(
                reqwest::Method::POST,
                "/mtrOrigemCompleto",
                &payload,
                "lr.obter_origem",
            )
            .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::ResourceNotFound,
                404,
                "MTR não encontrado",
                ErrorContext::new("lr.obter_origem"),
            )
        })
    }

    pub async fn obter_processo(
        &self,
        man_numero: &str,
    ) -> Result<serde_json::Value, MtrError> {
        let payload = MtrProcessoCompletoDTO {
            man_numero: man_numero.to_string(),
        };

        let raw: ObjetoRespostaRest<serde_json::Value> = self
            .client
            .request_with_body(
                reqwest::Method::POST,
                "/mtrProcesso",
                &payload,
                "lr.obter_processo",
            )
            .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::ResourceNotFound,
                404,
                "Processo não encontrado",
                ErrorContext::new("lr.obter_processo"),
            )
        })
    }

    pub async fn alterar_titularidade(
        &self,
        payload: &ApiwsAlteraManifestoOrigemDTO,
    ) -> Result<serde_json::Value, MtrError> {
        if payload.justificativa.len() < 20 {
            return Err(MtrError::validation(
                "Justificativa deve ter no mínimo 20 caracteres",
            ));
        }

        let raw: ObjetoRespostaRest<serde_json::Value> = self
            .client
            .request_with_body(
                reqwest::Method::POST,
                "/alterarManifestoLROrigem",
                payload,
                "lr.alterar_titularidade",
            )
            .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Alteração de titularidade falhou",
                ErrorContext::new("lr.alterar_titularidade"),
            )
        })
    }
}

pub(crate) fn validar_periodo(inicio: &DataMtr, fim: &DataMtr) -> Result<(), MtrError> {
    let diff = fim.0 - inicio.0;
    let dias = diff.num_days();
    if dias < 0 {
        return Err(MtrError::validation(
            "dataFinal deve ser posterior a dataInicial",
        ));
    }
    if dias > 30 {
        return Err(MtrError::validation(
            "Período máximo entre datas é de 30 dias",
        ));
    }
    Ok(())
}
