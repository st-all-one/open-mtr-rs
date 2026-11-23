use crate::download::safe_path;
use crate::error::CliError;
use open_mtr_core::openmtr::domain::DownloadItem;
use serde_json::{Value, json};
use std::io::Read;
use std::path::Path;

pub(super) fn error_code(e: &CliError) -> &'static str {
    match e {
        CliError::Config(_) => "config",
        CliError::Auth(_) => "auth",
        CliError::NotFound(_) => "not_found",
        CliError::Conflict(_) => "conflict",
        CliError::Network(_) => "network",
        CliError::Usage(_) => "usage",
        CliError::Partial(_) => "partial",
        CliError::Internal(_) => "internal",
    }
}

pub(super) fn read_input(file: Option<&Path>) -> Result<String, CliError> {
    let path = file.ok_or_else(|| CliError::Usage("batch exige --file <JSONL|->".into()))?;
    if path == Path::new("-") {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| CliError::Usage(e.to_string()))?;
        Ok(buf)
    } else {
        std::fs::read_to_string(path)
            .map_err(|e| CliError::Config(format!("ler {}: {e}", path.display())))
    }
}

pub(super) fn serialize_value<T: serde::Serialize>(value: T) -> Value {
    // Tipos de resposta da API são estritamente serializáveis: um erro aqui
    // indicaria um bug do adapter, não uma condição recuperável pela CLI.
    #[allow(
        clippy::expect_used,
        reason = "serialização de tipo interno é infalível por contrato"
    )]
    serde_json::to_value(value).expect("resposta da API serializável")
}

pub(super) fn write_downloads(out: &Path, items: Vec<DownloadItem>) -> Result<Value, CliError> {
    std::fs::create_dir_all(out).map_err(|e| CliError::Config(e.to_string()))?;
    let mut files = Vec::new();
    for item in items {
        let path = safe_path(out, &item.filename);
        std::fs::write(&path, &item.buffer).map_err(|e| CliError::Config(e.to_string()))?;
        files.push(path.display().to_string());
    }
    Ok(json!({ "files": files }))
}
