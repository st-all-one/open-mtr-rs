use serde::{Deserialize, Serialize};

/// Classificação tipada de um [`OpenMTRError`](super::OpenMTRError).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ErrorKind {
    /// Falha de comunicação com a rede (status 503).
    Network,
    /// Tempo limite excedido (status 504); conta como rede em [`OpenMTRError::is_network`](super::OpenMTRError::is_network).
    Timeout,
    /// Credenciais inválidas ou ausentes (status 401).
    Auth,
    /// Dados de entrada inválidos (status 422).
    Validation,
    /// Configuração inválida (status 500).
    Config,
    /// Erro genérico devolvido pela API (status arbitrário).
    Api,
    /// Recurso não encontrado (status 404).
    NotFound,
    /// Operação não suportada pelo sistema (status 501).
    NotSupported,
    #[default]
    /// Erro interno; variante padrão quando não há classificação mais específica.
    Internal,
}
