//! Operações SINIR: autenticação.

use super::SinirAdapter;

use tracing::{info, instrument};

use crate::openmtr::adapters::MtrCtx;
use crate::openmtr::error::OpenMTRError;

impl SinirAdapter {
    #[instrument(name = "sinir.autenticar_token", skip_all)]
    /// Executa a operação SINIR `autenticar_token`.
    pub(super) async fn autenticar_token_impl(&self, ctx: &MtrCtx) -> Result<String, OpenMTRError> {
        ctx.http.set_refresh_url(&ctx.config.endpoint("/token"));

        // O token do .env é um *refresh token* (tipo apiws-refresh, ~1 ano de
        // validade). O SINIR exige um *access token* (tipo apiws, ~8h) obtido
        // via POST /token. Se não fizermos o refresh aqui, o refresh token é
        // usado diretamente como access token e a API responde 401.
        ctx.http.refresh_token().await?;

        let token = ctx.http.get_token().await?;
        info!("Token SINIR renovado com sucesso");
        Ok(token)
    }

    /// Executa a operação SINIR `autenticar_credenciais`.
    pub(super) fn autenticar_credenciais_impl(
        _ctx: &MtrCtx,
        _params: &crate::openmtr::open_mtr_auth::AuthParams,
    ) -> Result<String, OpenMTRError> {
        Err(OpenMTRError::not_supported(
            "SINIR usa integration_token, não CPF/senha",
        ))
    }
}
