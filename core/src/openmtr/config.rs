use std::time::Duration;

use url::Url;

use crate::openmtr::domain::newtypes::Cpf;
use crate::openmtr::system::System;

use super::error::OpenMTRError;

#[derive(Debug, Clone)]
pub struct Config {
    pub system: System,
    pub base_url: String,
    pub integration_token: Option<String>,
    pub timeout: Duration,
    /// Limite de requisições simultâneas usado pelo cliente HTTP.
    ///
    /// O nome histórico é `max_rps`; na prática ele limita a concorrência de
    /// rede para evitar estourar o rate limit das APIs estaduais.
    pub max_rps: u32,
    pub cnpj: Option<String>,
}

impl Config {
    /// Monta a URL absoluta de um endpoint, normalizando a barra inicial.
    #[must_use]
    pub fn endpoint(&self, path: &str) -> String {
        if path.is_empty() {
            return self.base_url.clone();
        }
        if path.starts_with('/') {
            format!("{}{path}", self.base_url)
        } else {
            format!("{}/{path}", self.base_url)
        }
    }

    /// Limite efetivo de concorrência (nunca zero).
    #[must_use]
    pub fn max_concurrency(&self) -> usize {
        usize::try_from(self.max_rps.max(1)).unwrap_or(1)
    }
}

impl Config {
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder {
            system: System::Sinir,
            base_url: None,
            integration_token: None,
            timeout: Duration::from_secs(30),
            max_rps: 10,
            cnpj: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigBuilder {
    system: System,
    base_url: Option<String>,
    integration_token: Option<String>,
    timeout: Duration,
    max_rps: u32,
    cnpj: Option<String>,
}

impl ConfigBuilder {
    #[must_use]
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

    #[must_use]
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    #[must_use]
    pub fn max_rps(mut self, rps: u32) -> Self {
        self.max_rps = rps;
        self
    }

    pub fn cnpj(mut self, cnpj: impl Into<String>) -> Self {
        self.cnpj = Some(cnpj.into());
        self
    }

    pub fn build(self) -> Result<Config, OpenMTRError> {
        let base_url = self
            .base_url
            .unwrap_or_else(|| self.system.default_base_url().to_string());

        if base_url.trim().is_empty() {
            return Err(OpenMTRError::config("base_url cannot be empty"));
        }

        let parsed = Url::parse(&base_url)
            .map_err(|e| OpenMTRError::config(format!("base_url inválida: {e}")))?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(OpenMTRError::config(
                "base_url deve usar o esquema http ou https",
            ));
        }

        let base_url = base_url.trim_end_matches('/').to_string();

        if self.max_rps == 0 {
            return Err(OpenMTRError::config("max_rps deve ser maior que zero"));
        }

        let config = Config {
            system: self.system,
            base_url,
            integration_token: self.integration_token,
            timeout: self.timeout,
            max_rps: self.max_rps,
            cnpj: self.cnpj,
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
    pub cnpj: String,
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
    fn test_config_builder_rejects_invalid_base_url() {
        let result = Config::builder().base_url("not-a-url").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_builder_normalizes_trailing_slash() {
        let config = Config::builder()
            .base_url("https://example.com/api/")
            .build()
            .expect("config válida");
        assert_eq!(config.base_url, "https://example.com/api");
        assert_eq!(config.endpoint("/token"), "https://example.com/api/token");
        assert_eq!(config.endpoint("token"), "https://example.com/api/token");
    }

    #[test]
    fn test_config_builder_rejects_zero_max_rps() {
        let result = Config::builder().max_rps(0).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_credenciais() {
        let cpf = Cpf::new("12345678909").expect("valid cpf");
        let cred = Credenciais {
            cpf,
            senha: "secret".into(),
            unidade: 1,
            cnpj: String::new(),
        };
        assert_eq!(cred.cpf.as_str(), "12345678909");
        assert_eq!(cred.senha, "secret");
        assert_eq!(cred.unidade, 1);
    }
}
