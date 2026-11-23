//! Erros da CLI e seus códigos de saída.
//!
//! [`CliError::exit_code`](crate::error::CliError::exit_code) é o contrato usado pelo `main` e pelo batch: cada
//! variante mapeia para um código único, permitindo distinguir falhas de
//! configuração, autenticação, rede e execução parcial.

/// Falha da CLI, com mensagem e código de saída associados.
#[derive(Debug)]
pub enum CliError {
    /// Falha de configuração: arquivo ilegível, TOML inválido ou conteúdo rejeitado na validação.
    Config(String),
    /// Credenciais ausentes, inválidas ou recusadas pelo sistema de origem.
    Auth(String),
    /// Recurso solicitado não encontrado (por exemplo, uma conta por ID).
    NotFound(String),
    /// Conflito de estado: versão incompatível, checksum divergente ou retomada de outro período.
    Conflict(String),
    /// Falha de comunicação HTTP com a API de origem ou de release.
    Network(String),
    /// Uso incorreto da CLI: argumentos, datas, formatos ou combinações inválidas.
    Usage(String),
    /// Execução parcial: o número de janelas ou operações que falharam.
    Partial(usize),
    /// Erro interno inesperado, inclusive conversões de [`anyhow::Error`].
    Internal(String),
}

impl CliError {
    /// Código de saída do processo para esta falha: `2` uso, `3` configuração,
    /// `4` autenticação, `5` não encontrado, `6` conflito, `7` rede e `1` parcial/interno.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Config(_) => 3,
            Self::Auth(_) => 4,
            Self::NotFound(_) => 5,
            Self::Conflict(_) => 6,
            Self::Network(_) => 7,
            Self::Partial(_) | Self::Internal(_) => 1,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(m) => write!(f, "config: {m}"),
            Self::Auth(m) => write!(f, "auth: {m}"),
            Self::NotFound(m) => write!(f, "not_found: {m}"),
            Self::Conflict(m) => write!(f, "conflict: {m}"),
            Self::Network(m) => write!(f, "network: {m}"),
            Self::Usage(m) => write!(f, "usage: {m}"),
            Self::Partial(n) => write!(f, "partial: {n} erro(s) em janelas"),
            Self::Internal(m) => write!(f, "internal: {m}"),
        }
    }
}

impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e.to_string())
    }
}
