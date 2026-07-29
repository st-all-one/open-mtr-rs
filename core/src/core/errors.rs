use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorCategory {
    AuthenticationFailed,
    AuthorizationDenied,
    ResourceNotFound,
    ValidationError,
    Conflict,
    RateLimited,
    Timeout,
    NetworkError,
    ParseError,
    InternalError,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl ErrorCategory {
    pub fn from_status(status: u16) -> Option<Self> {
        match status {
            401 => Some(Self::AuthenticationFailed),
            403 => Some(Self::AuthorizationDenied),
            404 => Some(Self::ResourceNotFound),
            409 => Some(Self::Conflict),
            422 => Some(Self::ValidationError),
            429 => Some(Self::RateLimited),
            504 => Some(Self::Timeout),
            503 => Some(Self::NetworkError),
            _ => None,
        }
    }

    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::AuthenticationFailed => "authentication-failed",
            Self::AuthorizationDenied => "authorization-denied",
            Self::ResourceNotFound => "resource-not-found",
            Self::Conflict => "conflict",
            Self::ValidationError => "validation-error",
            Self::RateLimited => "rate-limited",
            Self::Timeout => "timeout",
            Self::NetworkError => "network-error",
            Self::ParseError => "parse-error",
            Self::InternalError => "internal-error",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    pub operation: Option<String>,
    pub http_status: Option<u16>,
    pub api_errors: Option<Vec<String>>,
    pub response_body: Option<String>,
}

impl ErrorContext {
    #[must_use]
    pub fn new(operation: &str) -> Self {
        Self {
            operation: Some(operation.into()),
            ..Default::default()
        }
    }
}

#[derive(Error, Debug)]
pub enum MtrError {
    #[error("[{category}] {detail} (instance: {instance})")]
    Api {
        category: ErrorCategory,
        status: u16,
        detail: String,
        instance: String,
        context: Box<ErrorContext>,
    },

    #[error("erro HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("rate limited (retry after: {retry_after:?})")]
    RateLimited {
        retry_after: Option<u64>,
        context: Box<ErrorContext>,
    },

    #[error("timeout após {duration:?}")]
    Timeout {
        duration: Duration,
        context: Box<ErrorContext>,
    },

    #[error("erro de serialização: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("erro de configuração: {0}")]
    Config(String),
}

impl MtrError {
    pub fn api(
        category: ErrorCategory,
        status: u16,
        detail: impl Into<String>,
        context: ErrorContext,
    ) -> Self {
        Self::Api {
            category,
            status,
            detail: detail.into(),
            instance: Uuid::now_v7().to_string(),
            context: Box::new(context),
        }
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Api {
            category: ErrorCategory::ValidationError,
            status: 422,
            detail: msg.into(),
            instance: Uuid::now_v7().to_string(),
            context: Box::new(ErrorContext::default()),
        }
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn deserialize(msg: impl Into<String>) -> Self {
        Self::Api {
            category: ErrorCategory::ParseError,
            status: 500,
            detail: msg.into(),
            instance: Uuid::now_v7().to_string(),
            context: Box::new(ErrorContext::default()),
        }
    }

    pub fn timeout(duration: Duration, operation: &str) -> Self {
        Self::Timeout {
            duration,
            context: Box::new(ErrorContext::new(operation)),
        }
    }

    #[must_use]
    pub fn category(&self) -> Option<ErrorCategory> {
        match self {
            Self::Api { category, .. } => Some(*category),
            Self::RateLimited { .. } => Some(ErrorCategory::RateLimited),
            Self::Timeout { .. } => Some(ErrorCategory::Timeout),
            Self::Http(e) if e.is_timeout() => Some(ErrorCategory::Timeout),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Api { status, .. } => matches!(status, 429 | 500..=599),
            Self::RateLimited { .. } => true,
            Self::Http(e) if e.is_timeout() => true,
            Self::Timeout { .. } => true,
            _ => false,
        }
    }
}
