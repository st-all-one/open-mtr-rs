use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

/// Classificação tipada de um [`OpenMTRError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ErrorKind {
    Network,
    Timeout,
    Auth,
    Validation,
    Config,
    Api,
    NotFound,
    NotSupported,
    #[default]
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[allow(
    clippy::upper_case_acronyms,
    reason = "OpenMTR é o nome oficial do projeto e do erro público"
)]
#[error("[{status}] {title} — {detail} (type: {type}, instance: {instance})")]
pub struct OpenMTRError {
    pub r#type: Url,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub instance: Uuid,
    /// Classificação estável para decisões de retry/tratamento.
    #[serde(default)]
    pub kind: ErrorKind,
}

impl OpenMTRError {
    fn with_kind(
        status: u16,
        title: impl Into<String>,
        detail: impl Into<String>,
        kind: ErrorKind,
        r#type: Url,
    ) -> Self {
        let title = title.into();
        let detail = detail.into();
        let instance = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
        tracing::info!(%status, %title, %detail, %instance, ?kind, "OpenMTRError created");
        Self {
            r#type,
            title,
            status,
            detail,
            instance,
            kind,
        }
    }

    pub fn new(status: u16, title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::with_kind(
            status,
            title,
            detail,
            ErrorKind::Internal,
            error_types::internal(),
        )
    }

    pub fn network(detail: impl Into<String>) -> Self {
        Self::with_kind(
            503,
            "Erro de rede",
            detail,
            ErrorKind::Network,
            error_types::network(),
        )
    }

    pub fn timeout(detail: impl Into<String>) -> Self {
        Self::with_kind(
            504,
            "Tempo limite excedido",
            detail,
            ErrorKind::Timeout,
            error_types::timeout(),
        )
    }

    pub fn auth(detail: impl Into<String>) -> Self {
        Self::with_kind(
            401,
            "Erro de autenticação",
            detail,
            ErrorKind::Auth,
            error_types::auth(),
        )
    }

    pub fn config(detail: impl Into<String>) -> Self {
        Self::with_kind(
            500,
            "Erro de configuração",
            detail,
            ErrorKind::Config,
            error_types::config(),
        )
    }

    pub fn validation(detail: impl Into<String>) -> Self {
        Self::with_kind(
            422,
            "Erro de validação",
            detail,
            ErrorKind::Validation,
            error_types::validation(),
        )
    }

    #[must_use]
    pub fn not_supported(operation: &str) -> Self {
        Self::with_kind(
            501,
            "Operação não suportada",
            format!("Operação não suportada: {operation}"),
            ErrorKind::NotSupported,
            error_types::not_supported(),
        )
    }

    pub fn parse_error(detail: impl Into<String>) -> Self {
        Self::validation(detail)
    }

    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::with_kind(
            404,
            "Não encontrado",
            detail,
            ErrorKind::NotFound,
            error_types::not_found(),
        )
    }

    pub fn api_error(status: u16, detail: impl Into<String>) -> Self {
        Self::with_kind(
            status,
            "Erro de API",
            detail,
            ErrorKind::Api,
            error_types::api(),
        )
    }

    /// Substitui o `instance` por um identificador determinístico.
    ///
    /// Útil para golden tests e para o `OpenMtrClient`, que reatribui as
    /// instâncias com o `IdGenerator` injetado no `MtrCtx`.
    #[must_use]
    pub fn with_instance(mut self, instance: Uuid) -> Self {
        self.instance = instance;
        self
    }

    #[must_use]
    pub fn is_timeout(&self) -> bool {
        matches!(self.kind, ErrorKind::Timeout) || self.status == 504
    }

    #[must_use]
    pub fn is_network(&self) -> bool {
        matches!(self.kind, ErrorKind::Network | ErrorKind::Timeout)
    }

    #[must_use]
    pub fn http_error(status: u16, title: &str, detail: &str) -> Self {
        let (kind, r#type) = match status {
            401 => (ErrorKind::Auth, error_types::auth()),
            404 => (ErrorKind::NotFound, error_types::not_found()),
            422 => (ErrorKind::Validation, error_types::validation()),
            _ => (ErrorKind::Api, error_types::api()),
        };
        Self::with_kind(status, title, detail, kind, r#type)
    }
}

#[cfg(feature = "native-http")]
impl From<reqwest::Error> for OpenMTRError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::timeout(err.to_string())
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
    use std::sync::LazyLock;
    use url::Url;

    #[allow(
        clippy::expect_used,
        reason = "URN montada apenas com sufixos literais conhecidos; a sintaxe é infalível"
    )]
    fn make_urn(suffix: &str) -> Url {
        Url::parse(&format!("urn:openmtr:error:{suffix}")).expect("URN de erro do openMTR é válida")
    }

    macro_rules! lazy_urn {
        ($name:ident, $suffix:literal) => {
            static $name: LazyLock<Url> = LazyLock::new(|| make_urn($suffix));
        };
    }

    lazy_urn!(NETWORK, "network");
    lazy_urn!(TIMEOUT, "timeout");
    lazy_urn!(AUTH, "authentication");
    lazy_urn!(VALIDATION, "validation");
    lazy_urn!(CONFIG, "configuration");
    lazy_urn!(NOT_FOUND, "not-found");
    lazy_urn!(API, "api-error");
    lazy_urn!(NOT_SUPPORTED, "not-supported");
    lazy_urn!(INTERNAL, "internal");

    #[must_use]
    pub fn network() -> Url {
        NETWORK.clone()
    }
    #[must_use]
    pub fn timeout() -> Url {
        TIMEOUT.clone()
    }
    #[must_use]
    pub fn auth() -> Url {
        AUTH.clone()
    }
    #[must_use]
    pub fn validation() -> Url {
        VALIDATION.clone()
    }
    #[must_use]
    pub fn config() -> Url {
        CONFIG.clone()
    }
    #[must_use]
    pub fn not_found() -> Url {
        NOT_FOUND.clone()
    }
    #[must_use]
    pub fn api() -> Url {
        API.clone()
    }
    #[must_use]
    pub fn not_supported() -> Url {
        NOT_SUPPORTED.clone()
    }
    #[must_use]
    pub fn internal() -> Url {
        INTERNAL.clone()
    }
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
        assert_eq!(err.kind, ErrorKind::Network);
    }

    #[test]
    fn test_timeout_error() {
        let err = OpenMTRError::timeout("crawled");
        assert_eq!(err.status, 504);
        assert_eq!(err.kind, ErrorKind::Timeout);
        assert!(err.is_timeout());
    }

    #[test]
    fn test_is_timeout_is_not_substring_based() {
        let err = OpenMTRError::network("upstream timeout announced");
        assert!(!err.is_timeout());
    }

    #[test]
    fn test_with_instance_is_deterministic() {
        let instance = Uuid::from_u128(42);
        let err = OpenMTRError::validation("x").with_instance(instance);
        assert_eq!(err.instance, instance);
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
        assert!(
            error_types::network()
                .as_str()
                .starts_with("urn:openmtr:error:")
        );
        assert!(
            error_types::timeout()
                .as_str()
                .starts_with("urn:openmtr:error:")
        );
        assert!(
            error_types::auth()
                .as_str()
                .starts_with("urn:openmtr:error:")
        );
        assert!(
            error_types::validation()
                .as_str()
                .starts_with("urn:openmtr:error:")
        );
        assert!(
            error_types::config()
                .as_str()
                .starts_with("urn:openmtr:error:")
        );
        assert!(
            error_types::not_found()
                .as_str()
                .starts_with("urn:openmtr:error:")
        );
        assert!(
            error_types::api()
                .as_str()
                .starts_with("urn:openmtr:error:")
        );
        assert!(
            error_types::not_supported()
                .as_str()
                .starts_with("urn:openmtr:error:")
        );
        assert!(
            error_types::internal()
                .as_str()
                .starts_with("urn:openmtr:error:")
        );
    }

    #[test]
    fn test_display() {
        let err = OpenMTRError::new(400, "x", "y");
        let s = err.to_string();
        assert!(s.contains("400"));
        assert!(s.contains('x'));
        assert!(s.contains('y'));
    }

    #[cfg(feature = "native-http")]
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
