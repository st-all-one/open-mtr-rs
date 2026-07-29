use std::time::{Duration, Instant, SystemTime};
use tracing::info;

pub(crate) struct TokenManager {
    integration_token: String,
    cached: Option<CachedToken>,
    refresh_url: Option<String>,
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

impl TokenManager {
    pub fn new(integration_token: impl Into<String>) -> Self {
        Self {
            integration_token: integration_token.into(),
            cached: None,
            refresh_url: None,
        }
    }

    pub fn set_refresh_url(&mut self, url: impl Into<String>) {
        self.refresh_url = Some(url.into());
    }

    pub fn token(&self) -> Option<&str> {
        self.cached.as_ref().and_then(|cached| {
            if Instant::now() < cached.expires_at {
                Some(cached.access_token.as_str())
            } else {
                None
            }
        })
    }

    pub fn extract_expiry(jwt: &str) -> Option<u64> {
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() < 2 {
            return None;
        }
        use base64::Engine;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let payload = engine.decode(parts[1]).ok()?;
        let claims: serde_json::Value = serde_json::from_slice(&payload).ok()?;
        claims.get("exp")?.as_u64()
    }

    pub async fn refresh(
        &mut self,
        http: &reqwest::Client,
    ) -> Result<(), crate::openmtr::error::OpenMTRError> {
        let url = self.refresh_url.as_deref().ok_or_else(|| {
            crate::openmtr::error::OpenMTRError::network("Refresh URL not set")
        })?;

        let resp = http
            .post(url)
            .header("Authorization", format!("Bearer {}", self.integration_token))
            .send()
            .await
            .map_err(|e| crate::openmtr::error::OpenMTRError::network(e.to_string()))?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await.map_err(|e| {
            crate::openmtr::error::OpenMTRError::parse_error(e.to_string())
        })?;

        if !status.is_success() {
            return Err(crate::openmtr::error::OpenMTRError::http_error(
                status.as_u16(),
                "Token refresh failed",
                "",
            ));
        }

        let has_error = body["erro"].as_bool().unwrap_or(true);
        if has_error {
            return Err(crate::openmtr::error::OpenMTRError::http_error(
                status.as_u16(),
                "Token refresh returned error",
                body["detail"].as_str().unwrap_or(""),
            ));
        }

        let objeto_resposta = body["objetoResposta"].as_str().ok_or_else(|| {
            crate::openmtr::error::OpenMTRError::parse_error(
                "Missing objetoResposta in refresh response",
            )
        })?;

        let token = objeto_resposta.strip_prefix("Bearer ").ok_or_else(|| {
            crate::openmtr::error::OpenMTRError::parse_error(
                "objetoResposta does not start with 'Bearer '",
            )
        })?;

        let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let ttl = if let Some(exp_unix) = Self::extract_expiry(token) {
            exp_unix.saturating_sub(now_secs)
        } else {
            8 * 60 * 60
        };
        let safe = ttl.saturating_sub(60);
        let expires_at = Instant::now() + Duration::from_secs(safe);

        self.cached = Some(CachedToken {
            access_token: token.to_string(),
            expires_at,
        });

        info!("Token refreshed successfully");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn integration_token(&self) -> &str {
        &self.integration_token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_expiry_valid_jwt() {
        use base64::Engine;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header = engine.encode(r#"{"alg":"HS256"}"#);
        let payload = engine.encode(r#"{"exp":2000000000}"#);
        let jwt = format!("{header}.{payload}.signature");
        assert_eq!(TokenManager::extract_expiry(&jwt), Some(2000000000));
    }

    #[test]
    fn test_extract_expiry_no_exp() {
        use base64::Engine;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header = engine.encode(r#"{"alg":"HS256"}"#);
        let payload = engine.encode(r#"{"sub":"test"}"#);
        let jwt = format!("{header}.{payload}.signature");
        assert!(TokenManager::extract_expiry(&jwt).is_none());
    }

    #[test]
    fn test_extract_expiry_invalid() {
        assert!(TokenManager::extract_expiry("not-a-jwt").is_none());
        assert!(TokenManager::extract_expiry("only.one.dot").is_none());
        assert!(TokenManager::extract_expiry("").is_none());
    }

    #[test]
    fn test_token_manager_initial_state() {
        let mgr = TokenManager::new("test-integration-token");
        assert_eq!(mgr.integration_token(), "test-integration-token");
        assert!(mgr.token().is_none());
    }

    #[test]
    fn test_set_refresh_url() {
        let mut mgr = TokenManager::new("token");
        mgr.set_refresh_url("https://example.com/refresh");
        assert!(mgr.token().is_none());
    }
}
