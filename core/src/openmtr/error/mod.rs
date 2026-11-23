mod conversions;
mod kind;
#[cfg(test)]
mod tests;
mod types;

pub use kind::ErrorKind;
pub use types::error_types;

use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

/// Erro estruturado do openMTR, no formato RFC 7807 (Problem Details).
///
/// Os construtores (`network`, `timeout`, `auth`, ...) fixam o par
/// status/título/URN/`kind`; [`Self::http_error`] faz o mapeamento de status
/// HTTP para `kind`. `instance` é um UUIDv7 gerado a cada erro, substituível
/// por [`Self::with_instance`].
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[allow(
    clippy::upper_case_acronyms,
    reason = "OpenMTR é o nome oficial do projeto e do erro público"
)]
#[error("[{status}] {title} — {detail} (type: {type}, instance: {instance})")]
pub struct OpenMTRError {
    /// URN do tipo de problema (`urn:openmtr:error:*`), serializado como string.
    pub r#type: Url,
    /// Título curto e legível do erro.
    pub title: String,
    /// Status HTTP associado ao erro.
    pub status: u16,
    /// Descrição detalhada da falha.
    pub detail: String,
    /// Identificador único do erro (UUIDv7); use [`Self::with_instance`] para fixá-lo.
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

    /// Cria um erro interno genérico com status, título e detalhe arbitrários.
    ///
    /// O `kind` é [`ErrorKind::Internal`] e o URN é `error_types::internal()`.
    pub fn new(status: u16, title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::with_kind(
            status,
            title,
            detail,
            ErrorKind::Internal,
            error_types::internal(),
        )
    }

    /// Cria um erro de rede com status 503 e URN de rede.
    pub fn network(detail: impl Into<String>) -> Self {
        Self::with_kind(
            503,
            "Erro de rede",
            detail,
            ErrorKind::Network,
            error_types::network(),
        )
    }

    /// Cria um erro de tempo limite com status 504 e URN de timeout.
    pub fn timeout(detail: impl Into<String>) -> Self {
        Self::with_kind(
            504,
            "Tempo limite excedido",
            detail,
            ErrorKind::Timeout,
            error_types::timeout(),
        )
    }

    /// Cria um erro de autenticação com status 401 e URN de autenticação.
    pub fn auth(detail: impl Into<String>) -> Self {
        Self::with_kind(
            401,
            "Erro de autenticação",
            detail,
            ErrorKind::Auth,
            error_types::auth(),
        )
    }

    /// Cria um erro de configuração com status 500 e URN de configuração.
    pub fn config(detail: impl Into<String>) -> Self {
        Self::with_kind(
            500,
            "Erro de configuração",
            detail,
            ErrorKind::Config,
            error_types::config(),
        )
    }

    /// Cria um erro de validação com status 422 e URN de validação.
    pub fn validation(detail: impl Into<String>) -> Self {
        Self::with_kind(
            422,
            "Erro de validação",
            detail,
            ErrorKind::Validation,
            error_types::validation(),
        )
    }

    /// Cria um erro de operação não suportada com status 501.
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

    /// Cria um erro de validação para falha de parse; equivale a [`Self::validation`].
    pub fn parse_error(detail: impl Into<String>) -> Self {
        Self::validation(detail)
    }

    /// Cria um erro de recurso não encontrado com status 404.
    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::with_kind(
            404,
            "Não encontrado",
            detail,
            ErrorKind::NotFound,
            error_types::not_found(),
        )
    }

    /// Cria um erro de API com o status informado e URN de erro de API.
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

    /// Indica timeout quando `kind` é [`ErrorKind::Timeout`] ou o status é 504.
    ///
    /// Usado pelo cliente HTTP para decidir retry; veja também [`Self::is_network`].
    #[must_use]
    pub fn is_timeout(&self) -> bool {
        matches!(self.kind, ErrorKind::Timeout) || self.status == 504
    }

    /// Indica falha de rede, incluindo timeouts.
    ///
    /// Verdadeiro para [`ErrorKind::Network`] e [`ErrorKind::Timeout`].
    #[must_use]
    pub fn is_network(&self) -> bool {
        matches!(self.kind, ErrorKind::Network | ErrorKind::Timeout)
    }

    /// Constrói um erro a partir de um status HTTP, mapeando-o para um `kind`.
    ///
    /// 401 → [`ErrorKind::Auth`], 404 → [`ErrorKind::NotFound`],
    /// 422 → [`ErrorKind::Validation`]; qualquer outro status → [`ErrorKind::Api`].
    /// O URN acompanha o `kind` escolhido.
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
