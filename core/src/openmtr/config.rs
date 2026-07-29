#![allow(clippy::module_name_repetitions)]

use std::time::Duration;

use crate::openmtr::domain::newtypes::Cpf;
use crate::openmtr::system::System;

use super::error::OpenMTRError;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub system: System,
    pub base_url: String,
    pub integration_token: Option<String>,
    pub timeout: Duration,
    #[allow(dead_code)]
    pub max_rps: u32,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder {
            system: System::Sinir,
            base_url: None,
            integration_token: None,
            timeout: Duration::from_secs(30),
            max_rps: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ConfigBuilder {
    system: System,
    base_url: Option<String>,
    integration_token: Option<String>,
    timeout: Duration,
    max_rps: u32,
}

impl ConfigBuilder {
    pub fn system(mut self, s: System) -> Self {
        self.system = s;
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.integration_token = Some(token.into());
        self
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    pub fn max_rps(mut self, rps: u32) -> Self {
        self.max_rps = rps;
        self
    }

    pub fn build(self) -> Result<Config, OpenMTRError> {
        let base_url = self
            .base_url
            .unwrap_or_else(|| self.system.default_base_url().to_string());

        if base_url.is_empty() {
            return Err(OpenMTRError::config("base_url cannot be empty"));
        }

        let config = Config {
            system: self.system,
            base_url,
            integration_token: self.integration_token,
            timeout: self.timeout,
            max_rps: self.max_rps,
        };

        tracing::info!(system = %config.system, base_url = %config.base_url, "Config built");

        Ok(config)
    }
}

#[derive(Debug, Clone)]
pub struct Credenciais {
    pub cpf: Cpf,
    pub senha: String,
    pub unidade: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder_defaults() {
        let config = Config::builder()
            .build()
            .expect("default config should build");
        assert_eq!(config.system, System::Sinir);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_rps, 10);
    }

    #[test]
    fn test_config_builder_custom() {
        let config = Config::builder()
            .system(System::Sigor)
            .base_url("https://example.com")
            .token("abc123")
            .timeout_secs(60)
            .max_rps(5)
            .build()
            .expect("custom config should build");
        assert_eq!(config.system, System::Sigor);
        assert_eq!(config.base_url, "https://example.com");
        assert_eq!(config.integration_token, Some("abc123".into()));
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.max_rps, 5);
    }

    #[test]
    fn test_config_builder_default_base_url() {
        let config = Config::builder()
            .system(System::Sinir)
            .build()
            .expect("sinir config should build");
        assert_eq!(config.base_url, System::Sinir.default_base_url());
    }

    #[test]
    fn test_config_builder_empty_base_url_fails() {
        let result = Config::builder().base_url("").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_credenciais() {
        let cpf = Cpf::new("12345678909").expect("valid cpf");
        let cred = Credenciais { cpf: cpf.clone(), senha: "secret".into(), unidade: 1 };
        assert_eq!(cred.cpf.as_str(), "12345678909");
        assert_eq!(cred.senha, "secret");
        assert_eq!(cred.unidade, 1);
    }
}
