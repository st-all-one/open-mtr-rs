//! Autenticação Bearer: renovação de token com single-flight e cache.

use crate::openmtr::error::OpenMTRError;
use crate::openmtr::transport::HttpRequest;

use super::MtrHttpClient;

impl MtrHttpClient {
    /// Substitui o access token em cache, preservando o refresh token configurado.
    ///
    /// O TTL é inferido do claim `exp` do JWT via
    /// [`TokenManager::extract_expiry`](crate::openmtr::client::token::TokenManager::extract_expiry),
    /// com fallback de 8 horas.
    pub fn set_token_direct(&self, jwt: &str) {
        let mut mgr = self
            .token_mgr
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Preserva o token de integração/refresh já configurado; apenas o access
        // token em cache é substituído.
        mgr.set_cached_token(jwt.to_string());
    }

    /// Define a URL usada para renovar o access token.
    ///
    /// Sem essa URL, a renovação automática falha com [`OpenMTRError::network`].
    pub fn set_refresh_url(&self, url: &str) {
        let mut mgr = self
            .token_mgr
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        mgr.set_refresh_url(url);
    }

    /// Renova explicitamente o access token com o refresh token, de forma serializada.
    ///
    /// Usa o mesmo mutex de single-flight das renovações disparadas por `401`.
    pub async fn refresh_token(&self) -> Result<(), OpenMTRError> {
        let _guard = self.refresh_lock.lock().await;
        self.refresh_token_locked().await
    }

    pub(super) async fn refresh_token_for(&self, stale_token: &str) -> Result<(), OpenMTRError> {
        let _guard = self.refresh_lock.lock().await;
        {
            let mgr = self
                .token_mgr
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(current) = mgr.token()
                && current != stale_token
            {
                // Outra requisição concorrente já renovou o access token.
                return Ok(());
            }
        }
        self.refresh_token_locked().await
    }

    async fn refresh_token_locked(&self) -> Result<(), OpenMTRError> {
        let refresh_req = {
            let mgr = self
                .token_mgr
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            mgr.force_refresh_data()
        };
        let (url, integration_token) = refresh_req
            .ok_or_else(|| OpenMTRError::network("Refresh URL not set or token not needed"))?;

        let req =
            HttpRequest::post(&url).header("Authorization", format!("Bearer {integration_token}"));
        let resp = self.transport.send(req).await?;

        if !resp.is_success() {
            return Err(OpenMTRError::http_error(
                resp.status,
                "Token refresh failed",
                "",
            ));
        }

        let body: serde_json::Value = resp.json()?;
        if body
            .get("erro")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true)
        {
            return Err(OpenMTRError::http_error(
                resp.status,
                "Token refresh returned error",
                body.get("detail")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
            ));
        }

        let objeto_resposta = body
            .get("objetoResposta")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                OpenMTRError::parse_error("Missing objetoResposta in refresh response")
            })?;
        let token = objeto_resposta.strip_prefix("Bearer ").ok_or_else(|| {
            OpenMTRError::parse_error("objetoResposta does not start with 'Bearer '")
        })?;

        let mut mgr = self
            .token_mgr
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        mgr.set_cached_token(token.to_string());
        Ok(())
    }

    /// Devolve o access token válido, renovando-o se necessário.
    ///
    /// A renovação é protegida por single-flight; se nenhum token ficar
    /// disponível, falha com [`OpenMTRError::auth`].
    pub async fn get_token(&self) -> Result<String, OpenMTRError> {
        {
            let guard = self
                .token_mgr
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(t) = guard.token() {
                return Ok(t.to_string());
            }
        }

        self.refresh_token_if_needed().await?;

        let guard = self
            .token_mgr
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard
            .token()
            .ok_or_else(|| OpenMTRError::auth("No token available after refresh"))
            .map(ToString::to_string)
    }

    async fn refresh_token_if_needed(&self) -> Result<(), OpenMTRError> {
        let _guard = self.refresh_lock.lock().await;
        {
            let mgr = self
                .token_mgr
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if mgr.token().is_some() {
                return Ok(());
            }
        }
        self.refresh_token_locked().await
    }
}
