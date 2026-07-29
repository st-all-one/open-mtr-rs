use crate::client::http::MtrClient;
use crate::core::errors::{ErrorCategory, ErrorContext, MtrError};
use crate::schemas::auth::{ApiwsLoginDTO, TokenObjeto, TokenResponse};

#[derive(Clone)]
pub struct AuthService {
    client: MtrClient,
}

impl AuthService {
    #[must_use]
    pub fn new(client: MtrClient) -> Self {
        Self { client }
    }

    #[deprecated(since = "0.1.0", note = "Use MtrClient com integration_token")]
    pub async fn autenticar(&self, creds: &ApiwsLoginDTO) -> Result<TokenObjeto, MtrError> {
        let resp: TokenResponse = self
            .client
            .request_with_body(reqwest::Method::POST, "/autenticar", creds, "auth.autenticar")
            .await?;

        if resp.erro || resp.objeto.is_none() {
            return Err(MtrError::api(
                ErrorCategory::AuthenticationFailed,
                401,
                &resp.mensagem,
                ErrorContext::new("auth.autenticar"),
            ));
        }

        Ok(resp.objeto.expect("objeto presente após validação"))
    }

    #[deprecated(since = "0.1.0", note = "Use MtrClient com integration_token")]
    pub async fn get_token(&self, creds: &ApiwsLoginDTO) -> Result<TokenObjeto, MtrError> {
        self.autenticar(creds).await
    }
}
