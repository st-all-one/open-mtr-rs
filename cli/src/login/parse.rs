use super::Login;
use crate::error::CliError;
use std::io::Read;

impl Login {
    /// Lê e valida o objeto de login.
    ///
    /// `-` lê o JSON do stdin; `@caminho` lê de um arquivo; qualquer outro
    /// valor é tratado como JSON inline.
    pub fn parse(raw: &str) -> Result<Self, CliError> {
        let text = if raw == "-" {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| CliError::Usage(format!("ler login do stdin: {e}")))?;
            buf
        } else if let Some(path) = raw.strip_prefix('@') {
            std::fs::read_to_string(path)
                .map_err(|e| CliError::Config(format!("ler login {path}: {e}")))?
        } else {
            raw.to_string()
        };
        let login: Self = serde_json::from_str(&text)
            .map_err(|e| CliError::Usage(format!("login inválido: {e}")))?;
        login.validate()?;
        Ok(login)
    }
}
