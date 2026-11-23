use crate::download::safe_path;
use crate::error::CliError;
use crate::output::OutputFormat;

/// Emite `value` na saída padrão no formato solicitado.
#[allow(clippy::print_stdout, reason = "CLI: a saída padrão é o produto")]
pub(super) fn print_value<T: serde::Serialize>(
    format: OutputFormat,
    value: &T,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Json => crate::output::print_json(value),
        OutputFormat::Text => {
            let v = serde_json::to_value(value).map_err(|e| CliError::Internal(e.to_string()))?;
            println!("{v}");
        }
    }
    Ok(())
}

pub(super) fn print_one_or_many(
    format: OutputFormat,
    results: Vec<serde_json::Value>,
) -> Result<(), CliError> {
    match results.as_slice() {
        [only] => print_value(format, only.get("data").unwrap_or(&serde_json::Value::Null)),
        _ => print_value(format, &results),
    }
}

pub(super) fn write_downloads(
    items: Vec<open_mtr_core::openmtr::domain::DownloadItem>,
    out: &std::path::Path,
    format: OutputFormat,
) -> Result<(), CliError> {
    std::fs::create_dir_all(out).map_err(|e| CliError::Config(format!("criar saída: {e}")))?;
    let mut files = Vec::new();
    for item in items {
        let path = safe_path(out, &item.filename);
        std::fs::write(&path, &item.buffer)
            .map_err(|e| CliError::Config(format!("escrever {}: {e}", path.display())))?;
        files.push(serde_json::json!({
            "file": path.display().to_string(),
            "format": item.formato,
            "size": item.buffer.len(),
        }));
    }
    print_value(format, &files)
}
