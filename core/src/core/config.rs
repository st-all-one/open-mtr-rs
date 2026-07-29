use crate::core::errors::MtrError;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum AuthMethod {
    #[default]
    BearerToken,
}

#[derive(Clone)]
pub struct MtrConfig {
    pub base_url: String,
    pub integration_token: String,
    pub auth_method: AuthMethod,
    pub timeout: Duration,
    pub max_rps: u32,
}

impl Default for MtrConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            integration_token: String::new(),
            auth_method: AuthMethod::default(),
            timeout: Duration::from_secs(60),
            max_rps: 10,
        }
    }
}

impl MtrConfig {
    pub fn builder() -> MtrConfigBuilder {
        MtrConfigBuilder::default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MtrConfigBuilder {
    base_url: Option<String>,
    integration_token: Option<String>,
    auth_method: Option<AuthMethod>,
    timeout: Option<Duration>,
    max_rps: Option<u32>,
}

impl MtrConfigBuilder {
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn integration_token(mut self, token: impl Into<String>) -> Self {
        self.integration_token = Some(token.into());
        self
    }

    pub fn auth_method(mut self, method: AuthMethod) -> Self {
        self.auth_method = Some(method);
        self
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Some(Duration::from_secs(secs));
        self
    }

    pub fn max_rps(mut self, rps: u32) -> Self {
        self.max_rps = Some(rps);
        self
    }

    pub fn build(self) -> Result<MtrConfig, MtrError> {
        let base_url = self
            .base_url
            .ok_or_else(|| MtrError::config("base_url é obrigatória"))?;
        let base_url = base_url.trim_end_matches('/').to_string();
        let integration_token = self
            .integration_token
            .ok_or_else(|| MtrError::config("integration_token é obrigatório"))?;
        Ok(MtrConfig {
            base_url,
            integration_token,
            auth_method: self.auth_method.unwrap_or_default(),
            timeout: self.timeout.unwrap_or(Duration::from_secs(60)),
            max_rps: self.max_rps.unwrap_or(10),
        })
    }
}
