//! Autenticação do MTR-PE: token JWT e credenciais CPF/senha/unidade.

use serde_json::{Value, json};
use tracing::{info, instrument};

use crate::openmtr::adapters::MtrCtx;
use crate::openmtr::error::OpenMTRError;

use super::MtrPeAdapter;

impl MtrPeAdapter {
    /// Configura diretamente o token JWT da configuração.
    pub(super) fn autenticar_token_impl(ctx: &MtrCtx) -> Result<String, OpenMTRError> {
        let token = ctx
            .config
            .integration_token
            .as_deref()
            .ok_or_else(|| OpenMTRError::config("integration_token é o JWT do MTR-PE"))?;
        ctx.http.set_token_direct(token);
        info!("Token MTR-PE configurado diretamente (sem refresh)");
        Ok(token.to_string())
    }

    #[instrument(name = "mtr_pe.autenticar_credenciais", skip_all)]
    /// Autentica via `/gettoken` com CPF, senha, unidade e CNPJ.
    pub(super) async fn autenticar_credenciais_impl(
        &self,
        ctx: &MtrCtx,
        params: &crate::openmtr::open_mtr_auth::AuthParams,
    ) -> Result<String, OpenMTRError> {
        #[allow(unreachable_patterns, reason = "defensivo: enum #[non_exhaustive]")]
        let (cpf, senha, unidade, cnpj) = match params {
            crate::openmtr::open_mtr_auth::AuthParams::CpfSenhaUnidade {
                cpf,
                senha,
                unidade,
                cnpj,
            } => (cpf.clone(), senha.clone(), *unidade, cnpj.clone()),
            _ => {
                return Err(OpenMTRError::validation(
                    "MTR-PE exige AuthParams::CpfSenhaUnidade",
                ));
            }
        };

        let cnpj = cnpj
            .or_else(|| ctx.config.cnpj.clone())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                OpenMTRError::validation(
                    "MTR-PE exige CNPJ em AuthParams::CpfSenhaUnidade ou Config::cnpj",
                )
            })?;

        let body = json!({
            "pessoaCodigo": unidade,
            "pessoaCnpj": cnpj,
            "usuarioCpf": cpf,
            "senha": senha,
        });

        let response: Value = ctx
            .http
            .post_public("/gettoken", &body, "autenticar")
            .await?;

        let jwt = response
            .get("token")
            .and_then(|s| s.as_str())
            .ok_or_else(|| OpenMTRError::auth("Resposta do /gettoken não contém token"))?;

        info!("Token MTR-PE obtido com sucesso");
        Ok(jwt.to_string())
    }
}
