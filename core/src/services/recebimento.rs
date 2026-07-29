use crate::client::http::MtrClient;
use crate::client::retry::with_retry;
use crate::core::errors::{ErrorCategory, ErrorContext, MtrError};
use crate::schemas::common::ObjetoRespostaRest;
use crate::schemas::recebimento::{
    AceiteAlteracaoDTO, AceiteAlteracaoResponse, ApiwsAlteraRecebimentoDTO,
    ApiwsManifestoDeRecebimentoDTO, RecebimentoLoteItem, ResiduoRecebido, TimestampMs,
};

#[derive(Clone)]
pub struct RecebimentoService {
    client: MtrClient,
}

impl RecebimentoService {
    #[must_use]
    pub fn new(client: MtrClient) -> Self {
        Self { client }
    }

    pub async fn registrar_lote(
        &self,
        itens: &[ApiwsManifestoDeRecebimentoDTO],
    ) -> Result<Vec<RecebimentoLoteItem>, MtrError> {
        if itens.is_empty() {
            return Err(MtrError::validation("Lista de recebimento vazia"));
        }

        let raw: ObjetoRespostaRest<Vec<RecebimentoLoteItem>> = with_retry(|| async {
            self.client
                .request_with_body(
                    reqwest::Method::POST,
                    "/receberManifestoLote",
                    itens,
                    "recebimento.registrar_lote",
                )
                .await
        })
        .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Recebimento falhou",
                ErrorContext::new("recebimento.registrar_lote"),
            )
        })
    }

    pub async fn solicitar_alteracao(
        &self,
        payload: &ApiwsAlteraRecebimentoDTO,
    ) -> Result<serde_json::Value, MtrError> {
        if payload.justificativa.len() < 10 {
            return Err(MtrError::validation(
                "Justificativa deve ter no mínimo 10 caracteres",
            ));
        }
        if payload.lista_residuos_alterados.is_empty() {
            return Err(MtrError::validation(
                "Lista de resíduos alterados não pode estar vazia",
            ));
        }

        let raw: ObjetoRespostaRest<serde_json::Value> = self
            .client
            .request_with_body(
                reqwest::Method::POST,
                "/alteraRecebimento",
                payload,
                "recebimento.solicitar_alteracao",
            )
            .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Alteração falhou",
                ErrorContext::new("recebimento.solicitar_alteracao"),
            )
        })
    }

    pub async fn aceitar_alteracao(
        &self,
        payload: &AceiteAlteracaoDTO,
    ) -> Result<AceiteAlteracaoResponse, MtrError> {
        let raw: ObjetoRespostaRest<AceiteAlteracaoResponse> = self
            .client
            .request_with_body(
                reqwest::Method::POST,
                "/aceiteAlteracaoRecebimento",
                payload,
                "recebimento.aceitar_alteracao",
            )
            .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Aceite falhou",
                ErrorContext::new("recebimento.aceitar_alteracao"),
            )
        })
    }
}

#[derive(Default)]
pub struct RecebimentoBuilder {
    man_numero: Option<String>,
    data_recebimento: Option<TimestampMs>,
    nome_responsavel: Option<String>,
    observacoes: Option<String>,
    residuos: Vec<ResiduoRecebido>,
}

impl RecebimentoBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn manifesto(mut self, num: impl Into<String>) -> Self {
        self.man_numero = Some(num.into());
        self
    }

    pub fn data(mut self, dt: TimestampMs) -> Self {
        self.data_recebimento = Some(dt);
        self
    }

    pub fn responsavel(mut self, nome: impl Into<String>) -> Self {
        self.nome_responsavel = Some(nome.into());
        self
    }

    pub fn observacoes(mut self, obs: impl Into<String>) -> Self {
        self.observacoes = Some(obs.into());
        self
    }

    pub fn add_residuo(
        mut self,
        codigo_ibama: impl Into<String>,
        quantidade: f64,
        unidade: i32,
    ) -> Self {
        self.residuos.push(ResiduoRecebido {
            res_codigo_ibama: codigo_ibama.into(),
            quantidade_recebida: quantidade,
            uni_codigo: unidade,
            tie_codigo: None,
            cla_codigo: None,
            tia_codigo: None,
            tra_codigo: None,
            sequencial: None,
        });
        self
    }

    pub fn build(self) -> Result<ApiwsManifestoDeRecebimentoDTO, MtrError> {
        Ok(ApiwsManifestoDeRecebimentoDTO {
            man_numero: self
                .man_numero
                .ok_or_else(|| MtrError::validation("man_numero é obrigatório"))?,
            data_recebimento: self
                .data_recebimento
                .ok_or_else(|| MtrError::validation("data_recebimento é obrigatório"))?,
            nome_responsavel: self
                .nome_responsavel
                .ok_or_else(|| MtrError::validation("nome_responsavel é obrigatório"))?,
            observacoes: self.observacoes,
            lista_residuos_recebidos: (!self.residuos.is_empty()).then_some(self.residuos),
        })
    }
}
