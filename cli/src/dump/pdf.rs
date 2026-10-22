use super::DumpError;
use crate::account::Account;
use crate::config::CliConfig;
use crate::error::CliError;
use crate::normalize::DumpRow;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::Path;

#[allow(
    clippy::excessive_nesting,
    reason = "laço conta -> linha -> download com tratamento de erro por item"
)]
pub(super) async fn download_pdfs(
    cfg: &CliConfig,
    accounts: &[&Account],
    rows: &[DumpRow],
    out: &Path,
    fixed_time: Option<chrono::DateTime<chrono::Utc>>,
    errors: &mut Vec<DumpError>,
) -> Result<(), CliError> {
    let dir = out.join("pdfs");
    std::fs::create_dir_all(&dir).map_err(|e| CliError::Config(format!("criar pdfs: {e}")))?;
    let mut manifest = String::new();
    for account in accounts {
        let mine: Vec<&DumpRow> = rows.iter().filter(|r| r.account == account.id).collect();
        if mine.is_empty() {
            continue;
        }
        let client = account.build_client(cfg, fixed_time).await?;
        for row in mine {
            let ident = if row.system == "mtr_pe" && !row.barcode.is_empty() {
                row.barcode.clone()
            } else {
                row.numero.clone()
            };
            match client.download_mtrs(vec![ident]).await {
                Ok(items) => {
                    for item in items.dados {
                        if !item.buffer.starts_with(b"%PDF") {
                            errors.push(DumpError {
                                account: account.id.clone(),
                                window: None,
                                code: "pdf_invalid".into(),
                                message: format!("PDF inválido: {}", item.filename),
                            });
                            continue;
                        }
                        let name =
                            format!("{}_{}.pdf", sanitize(&account.id), sanitize(&row.numero));
                        let path = dir.join(&name);
                        let hash = hex::encode(Sha256::digest(&item.buffer));
                        match std::fs::write(&path, &item.buffer) {
                            Ok(()) => {
                                let entry = json!({
                                    "account": account.id,
                                    "system": row.system,
                                    "numero": row.numero,
                                    "file": name,
                                    "sha256": hash,
                                    "size": item.buffer.len(),
                                    "status": "ok",
                                });
                                manifest.push_str(&entry.to_string());
                                manifest.push('\n');
                            }
                            Err(e) => errors.push(DumpError {
                                account: account.id.clone(),
                                window: None,
                                code: "pdf_io".into(),
                                message: e.to_string(),
                            }),
                        }
                    }
                }
                Err(e) => errors.push(DumpError {
                    account: account.id.clone(),
                    window: None,
                    code: "pdf".into(),
                    message: e.to_string(),
                }),
            }
        }
    }
    std::fs::write(dir.join("manifest.jsonl"), manifest)
        .map_err(|e| CliError::Config(format!("escrever manifesto de PDFs: {e}")))?;
    Ok(())
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
