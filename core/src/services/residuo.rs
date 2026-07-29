use crate::client::http::MtrClient;
use crate::core::errors::{ErrorCategory, ErrorContext, MtrError};
use crate::schemas::common::ObjetoRespostaRest;
use crate::schemas::residuo::ResiduoIbama;

#[derive(Clone)]
pub struct ResiduoService {
    client: MtrClient,
}

impl ResiduoService {
    #[must_use]
    pub fn new(client: MtrClient) -> Self {
        Self { client }
    }

    pub async fn listar_todos(&self) -> Result<Vec<ResiduoIbama>, MtrError> {
        let raw: ObjetoRespostaRest<Vec<ResiduoIbama>> = self
            .client
            .request(reqwest::Method::GET, "/retornaListaResiduo", "residuo.listar")
            .await?;

        raw.objeto_resposta.ok_or_else(|| {
            MtrError::api(
                ErrorCategory::InternalError,
                500,
                "Lista de resíduos vazia",
                ErrorContext::new("residuo.listar"),
            )
        })
    }

    pub async fn listar_perigosos(&self) -> Result<Vec<ResiduoIbama>, MtrError> {
        let todos = self.listar_todos().await?;
        Ok(todos.into_iter().filter(ResiduoIbama::is_perigoso).collect())
    }
}
