use crate::error::CliError;

pub(super) fn env_secret(field: &str, name: &str) -> Result<String, CliError> {
    let value = std::env::var(name)
        .map_err(|_| CliError::Config(format!("login: variável {name} ({field}) não definida")))?;
    if value.trim().is_empty() {
        return Err(CliError::Config(format!(
            "login: variável {name} ({field}) vazia"
        )));
    }
    Ok(value)
}
