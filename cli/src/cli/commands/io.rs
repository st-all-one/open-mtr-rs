use crate::error::CliError;

pub fn print_value<T: serde::Serialize>(json: bool, value: &T) -> Result<(), CliError> {
    if json {
        crate::output::print_json(value);
    } else {
        let v = serde_json::to_value(value).map_err(|e| CliError::Internal(e.to_string()))?;
        println!("{v}");
    }
    Ok(())
}

pub(super) fn print_one_or_many(
    json: bool,
    results: Vec<serde_json::Value>,
) -> Result<(), CliError> {
    if results.len() == 1 {
        print_value(json, &results[0]["data"])
    } else {
        print_value(json, &results)
    }
}

pub(super) fn write_downloads(
    items: Vec<open_mtr_core::openmtr::domain::DownloadItem>,
    out: &std::path::Path,
    json: bool,
) -> Result<(), CliError> {
    std::fs::create_dir_all(out).map_err(|e| CliError::Config(format!("criar saída: {e}")))?;
    let mut files = Vec::new();
    for item in items {
        let name = item
            .filename
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or("arquivo");
        let path = out.join(name);
        std::fs::write(&path, &item.buffer)
            .map_err(|e| CliError::Config(format!("escrever {}: {e}", path.display())))?;
        files.push(serde_json::json!({
            "file": path.display().to_string(),
            "format": item.formato,
            "size": item.buffer.len(),
        }));
    }
    print_value(json, &files)
}
