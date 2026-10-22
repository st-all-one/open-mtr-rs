use std::process::ExitCode;

#[derive(Debug)]
pub enum CliError {
    Config(String),
    Auth(String),
    NotFound(String),
    Conflict(String),
    Network(String),
    Usage(String),
    Partial(usize),
    Internal(String),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Usage(_) => 2,
            CliError::Config(_) => 3,
            CliError::Auth(_) => 4,
            CliError::NotFound(_) => 5,
            CliError::Conflict(_) => 6,
            CliError::Network(_) => 7,
            CliError::Partial(_) => 1,
            CliError::Internal(_) => 1,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Config(m) => write!(f, "config: {m}"),
            CliError::Auth(m) => write!(f, "auth: {m}"),
            CliError::NotFound(m) => write!(f, "not_found: {m}"),
            CliError::Conflict(m) => write!(f, "conflict: {m}"),
            CliError::Network(m) => write!(f, "network: {m}"),
            CliError::Usage(m) => write!(f, "usage: {m}"),
            CliError::Partial(n) => write!(f, "partial: {n} erro(s) em janelas"),
            CliError::Internal(m) => write!(f, "internal: {m}"),
        }
    }
}

impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        CliError::Internal(e.to_string())
    }
}

#[allow(unused)]
pub fn _exit() -> ExitCode {
    ExitCode::SUCCESS
}
