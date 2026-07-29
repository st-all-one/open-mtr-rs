#![allow(clippy::module_name_repetitions)]

use std::fmt;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OpenMTRError {
    pub r#type: Url,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub instance: Uuid,
}

impl OpenMTRError {
    pub fn new(status: u16, title: impl Into<String>, detail: impl Into<String>) -> Self {
        let title = title.into();
        let detail = detail.into();
        let instance = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
        tracing::info!(%status, %title, %detail, %instance, "OpenMTRError created");
        Self { r#type: error_types::internal(), title, status, detail, instance }
    }

    pub fn network(detail: impl Into<String>) -> Self {
        Self {
            r#type: error_types::network(),
            title: "Erro de rede".into(),
            status: 503,
            detail: detail.into(),
            instance: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
        }
    }

    pub fn timeout(detail: impl Into<String>) -> Self {
        Self {
            r#type: error_types::timeout(),
            title: "Tempo limite excedido".into(),
            status: 504,
            detail: detail.into(),
            instance: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
        }
    }

    pub fn auth(detail: impl Into<String>) -> Self {
        Self {
            r#type: error_types::auth(),
            title: "Erro de autenticação".into(),
            status: 401,
            detail: detail.into(),
            instance: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
        }
    }

    pub fn config(detail: impl Into<String>) -> Self {
        Self {
            r#type: error_types::config(),
            title: "Erro de configuração".into(),
            status: 500,
            detail: detail.into(),
            instance: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
        }
    }

    pub fn validation(detail: impl Into<String>) -> Self {
        Self {
            r#type: error_types::validation(),
            title: "Erro de validação".into(),
            status: 422,
            detail: detail.into(),
            instance: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
        }
    }

    pub fn not_supported(operation: &str) -> Self {
        Self {
            r#type: error_types::not_supported(),
            title: "Operação não suportada".into(),
            status: 501,
            detail: format!("Operação não suportada: {operation}"),
            instance: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
        }
    }

    pub fn parse_error(detail: impl Into<String>) -> Self {
        Self::validation(detail)
    }

    pub fn http_error(status: u16, title: &str, detail: &str) -> Self {
        Self {
            r#type: error_types::api(),
            title: title.to_string(),
            status,
            detail: detail.to_string(),
            instance: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
        }
    }
}

impl fmt::Display for OpenMTRError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} — {} (type: {}, instance: {})",
            self.status, self.title, self.detail, self.r#type, self.instance
        )
    }
}

impl std::error::Error for OpenMTRError {}

impl From<reqwest::Error> for OpenMTRError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::timeout(err.to_string())
        } else if err.is_connect() {
            Self::network(err.to_string())
        } else {
            Self::network(err.to_string())
        }
    }
}

impl From<serde_json::Error> for OpenMTRError {
    fn from(err: serde_json::Error) -> Self {
        Self::validation(err.to_string())
    }
}

pub mod error_types {
    use url::Url;

    fn make_urn(suffix: &str) -> Url {
        Url::parse(&format!("urn:openmtr:error:{suffix}"))
            .expect("urn:openmtr:error:{suffix} is a valid Url")
    }

    pub fn network() -> Url { make_urn("network") }
    pub fn timeout() -> Url { make_urn("timeout") }
    pub fn auth() -> Url { make_urn("authentication") }
    pub fn validation() -> Url { make_urn("validation") }
    pub fn config() -> Url { make_urn("configuration") }
    pub fn not_found() -> Url { make_urn("not-found") }
    pub fn api() -> Url { make_urn("api-error") }
    pub fn not_supported() -> Url { make_urn("not-supported") }
    pub fn internal() -> Url { make_urn("internal") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_error() {
        let err = OpenMTRError::new(400, "Bad Request", "detail");
        assert_eq!(err.status, 400);
        assert_eq!(err.title, "Bad Request");
    }

    #[test]
    fn test_network_error() {
        let err = OpenMTRError::network("connection refused");
        assert_eq!(err.status, 503);
    }

    #[test]
    fn test_timeout_error() {
        let err = OpenMTRError::timeout("crawled");
        assert_eq!(err.status, 504);
    }

    #[test]
    fn test_auth_error() {
        let err = OpenMTRError::auth("invalid token");
        assert_eq!(err.status, 401);
    }

    #[test]
    fn test_config_error() {
        let err = OpenMTRError::config("missing base_url");
        assert_eq!(err.status, 500);
    }

    #[test]
    fn test_validation_error() {
        let err = OpenMTRError::validation("invalid cpf");
        assert_eq!(err.status, 422);
    }

    #[test]
    fn test_not_supported() {
        let err = OpenMTRError::not_supported("export");
        assert_eq!(err.status, 501);
        assert!(err.detail.contains("export"));
    }

    #[test]
    fn test_error_types_are_valid_urls() {
        assert!(error_types::network().as_str().starts_with("urn:openmtr:error:"));
        assert!(error_types::timeout().as_str().starts_with("urn:openmtr:error:"));
        assert!(error_types::auth().as_str().starts_with("urn:openmtr:error:"));
        assert!(error_types::validation().as_str().starts_with("urn:openmtr:error:"));
        assert!(error_types::config().as_str().starts_with("urn:openmtr:error:"));
        assert!(error_types::not_found().as_str().starts_with("urn:openmtr:error:"));
        assert!(error_types::api().as_str().starts_with("urn:openmtr:error:"));
        assert!(error_types::not_supported().as_str().starts_with("urn:openmtr:error:"));
        assert!(error_types::internal().as_str().starts_with("urn:openmtr:error:"));
    }

    #[test]
    fn test_display() {
        let err = OpenMTRError::new(400, "x", "y");
        let s = err.to_string();
        assert!(s.contains("400"));
        assert!(s.contains("x"));
        assert!(s.contains("y"));
    }

    #[tokio::test]
    async fn test_from_reqwest_error() {
        let body = reqwest::Client::new()
            .get("http://127.0.0.1:1")
            .send()
            .await;
        if let Err(e) = body {
            let err: OpenMTRError = e.into();
            assert!(err.status == 503 || err.status == 504);
        }
    }
}
