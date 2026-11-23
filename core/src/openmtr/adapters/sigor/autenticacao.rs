//! Autenticação por token JWT e por credenciais no SIGOR.

use serde_json::{Value, json};
use tracing::{info, instrument};

use super::SigorAdapter;
use crate::openmtr::adapters::MtrCtx;
use crate::openmtr::error::OpenMTRError;

impl SigorAdapter {
    /// Configura o token de integração JWT fornecido diretamente.
    pub(super) fn autenticar_token_impl(ctx: &MtrCtx) -> Result<String, OpenMTRError> {
        let token = ctx
            .config
            .integration_token
            .as_deref()
            .ok_or_else(|| OpenMTRError::config("integration_token é o JWT do SIGOR"))?;
        ctx.http.set_token_direct(token);
        info!("Token SIGOR configurado diretamente (sem refresh)");
        Ok(token.to_string())
    }

    /// Obtém um JWT junto do endpoint `/gettoken` usando CPF, senha e unidade.
    #[instrument(name = "sigor.autenticar_credenciais", skip_all)]
    pub(super) async fn autenticar_credenciais_impl(
        &self,
        ctx: &MtrCtx,
        params: &crate::openmtr::open_mtr_auth::AuthParams,
    ) -> Result<String, OpenMTRError> {
        #[allow(unreachable_patterns, reason = "defensivo: enum #[non_exhaustive]")]
        let (cpf, senha, unidade) = match params {
            crate::openmtr::open_mtr_auth::AuthParams::CpfSenhaUnidade {
                cpf,
                senha,
                unidade,
                ..
            } => (cpf.clone(), senha.clone(), *unidade),
            _ => {
                return Err(OpenMTRError::validation(
                    "SIGOR exige AuthParams::CpfSenhaUnidade",
                ));
            }
        };

        let body = json!({
            "cpfCnpj": cpf,
            "senha": senha,
            "unidade": unidade.to_string(),
        });

        let response: Value = ctx
            .http
            .post_public("/gettoken", &body, "autenticar")
            .await?;

        let token_str = response
            .get("objetoResposta")
            .and_then(|s| s.as_str())
            .ok_or_else(|| OpenMTRError::auth("Resposta do /gettoken não contém objetoResposta"))?;

        let jwt = token_str.trim_prefix("Bearer ");

        info!("Token SIGOR obtido com sucesso");
        Ok(jwt.to_string())
    }
}
