use std::time::Duration;

#[cfg(target_arch = "wasm32")]
use web_time::Instant;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
use tracing::info;

pub struct TokenManager {
    pub integration_token: String,
    cached: Option<CachedToken>,
    pub refresh_url: Option<String>,
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

impl TokenManager {
    #[allow(dead_code)]
    pub fn new(integration_token: impl Into<String>) -> Self {
        Self {
            integration_token: integration_token.into(),
            cached: None,
            refresh_url: None,
        }
    }

    #[must_use]
    pub fn from_cached(access_token: String, expires_at: Instant) -> Self {
        Self {
            integration_token: access_token.clone(),
            cached: Some(CachedToken { access_token, expires_at }),
            refresh_url: None,
        }
    }

    pub fn set_refresh_url(&mut self, url: impl Into<String>) {
        self.refresh_url = Some(url.into());
    }

    #[must_use]
    pub fn token(&self) -> Option<&str> {
        self.cached.as_ref().and_then(|cached| {
            if Instant::now() < cached.expires_at {
                Some(cached.access_token.as_str())
            } else {
                None
            }
        })
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn needs_refresh(&self) -> bool {
        self.token().is_none()
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn refresh_data(&self) -> Option<(String, String)> {
        let url = self.refresh_url.clone()?;
        if self.needs_refresh() {
            Some((url, self.integration_token.clone()))
        } else {
            None
        }
    }

    #[must_use]
    pub fn force_refresh_data(&self) -> Option<(String, String)> {
        let url = self.refresh_url.clone()?;
        Some((url, self.integration_token.clone()))
    }

    pub fn set_cached_token(&mut self, access_token: String) {
        let now_secs = crate::openmtr::client::http::now_unix_secs();
        let ttl = if let Some(exp_unix) = Self::extract_expiry(&access_token) {
            exp_unix.saturating_sub(now_secs)
        } else {
            8 * 60 * 60
        };
        let safe = ttl.saturating_sub(60);
        let expires_at = Instant::now() + Duration::from_secs(safe);

        self.cached = Some(CachedToken {
            access_token,
            expires_at,
        });

        info!("Token refreshed successfully");
    }

    #[must_use]
    pub fn extract_expiry(jwt: &str) -> Option<u64> {
        use base64::Engine;
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() < 2 {
            return None;
        }
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let payload = engine.decode(parts[1]).ok()?;
        let claims: serde_json::Value = serde_json::from_slice(&payload).ok()?;
        claims.get("exp")?.as_u64()
    }

    #[allow(dead_code)]
    #[must_use]
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

    #[test]
    fn test_force_refresh_data_with_url() {
        let mut mgr = TokenManager::from_cached(
            "access-token".into(),
            std::time::Instant::now() + std::time::Duration::from_secs(3600),
        );
        mgr.set_refresh_url("https://example.com/token");
        let data = mgr.force_refresh_data().unwrap();
        assert_eq!(data.0, "https://example.com/token");
        assert_eq!(data.1, "access-token");
    }

    #[test]
    fn test_force_refresh_data_without_url() {
        let mgr = TokenManager::from_cached(
            "access-token".into(),
            std::time::Instant::now() + std::time::Duration::from_secs(3600),
        );
        assert!(mgr.force_refresh_data().is_none());
    }

    #[test]
    fn test_needs_refresh_when_valid() {
        let mgr = TokenManager::from_cached(
            "token".into(),
            std::time::Instant::now() + std::time::Duration::from_secs(3600),
        );
        assert!(!mgr.needs_refresh());
    }

    #[test]
    fn test_needs_refresh_when_expired() {
        let mgr = TokenManager::from_cached(
            "token".into(),
            std::time::Instant::now(),
        );
        assert!(mgr.needs_refresh());
    }

    #[test]
    fn test_set_cached_token() {
        use base64::Engine;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header = engine.encode(r#"{"alg":"HS256"}"#);
        let payload = engine.encode(r#"{"exp":3000000000}"#);
        let jwt = format!("{header}.{payload}.sig");

        let mut mgr = TokenManager::new("integration");
        assert!(mgr.token().is_none());

        mgr.set_cached_token(jwt.clone());
        assert!(mgr.token().is_some());
        assert_eq!(mgr.token().unwrap(), jwt);
    }
}
