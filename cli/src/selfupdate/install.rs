use super::{Plan, UpdateOptions};
use crate::error::CliError;
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};

pub(super) async fn install(plan: &Plan, opts: &UpdateOptions) -> Result<(), CliError> {
    let client = reqwest::Client::new();
    let checksums = client
        .get(&plan.checksums_url)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("baixar checksums: {e}")))?
        .text()
        .await
        .map_err(|e| CliError::Network(format!("ler checksums: {e}")))?;
    let expected = parse_checksums(&checksums, &plan.asset)
        .ok_or_else(|| CliError::Network(format!("checksum ausente para {}", plan.asset)))?;

    let bytes = client
        .get(&plan.asset_url)
        .send()
        .await
        .map_err(|e| CliError::Network(format!("baixar binário: {e}")))?
        .bytes()
        .await
        .map_err(|e| CliError::Network(format!("ler binário: {e}")))?;

    let actual = hex::encode(Sha256::digest(&bytes));
    if actual != expected {
        return Err(CliError::Conflict(format!(
            "checksum divergente para {}: esperado {expected}, obtido {actual}",
            plan.asset
        )));
    }

    if plan.target.contains("windows") {
        return Err(CliError::Usage(
            "self update: extração .zip no Windows ainda não suportada; instale manualmente".into(),
        ));
    }
    let binary = extract_tar_gz(&bytes)?;
    let install_path = install_path(opts)?;
    write_atomic(&install_path, &binary)?;
    Ok(())
}

fn install_path(opts: &UpdateOptions) -> Result<PathBuf, CliError> {
    if let Some(p) = &opts.install_path {
        return Ok(p.clone());
    }
    if let Ok(p) = std::env::var("OPENMTR_INSTALL_PATH") {
        return Ok(PathBuf::from(p));
    }
    std::env::current_exe().map_err(|e| CliError::Config(format!("descobrir binário atual: {e}")))
}

fn write_atomic(path: &Path, binary: &[u8]) -> Result<(), CliError> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let staging = dir.join(format!(
        ".{}.new.{}",
        path.file_name().map_or_else(
            || "openmtr".to_string(),
            |n| n.to_string_lossy().to_string(),
        ),
        std::process::id()
    ));
    std::fs::write(&staging, binary)
        .map_err(|e| CliError::Config(format!("gravar staging: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&staging, mode)
            .map_err(|e| CliError::Config(format!("chmod staging: {e}")))?;
    }
    std::fs::rename(&staging, path).map_err(|e| {
        if let Err(cleanup) = std::fs::remove_file(&staging) {
            tracing::debug!(%cleanup, "não foi possível limpar o staging");
        }
        CliError::Config(format!("instalar binário: {e}"))
    })
}

/// Extrai o SHA-256 de `text` para `asset` de um `sha256sums.txt`; aceita o prefixo `*` antes do nome.
pub fn parse_checksums(text: &str, asset: &str) -> Option<String> {
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next().unwrap_or_default().trim_start_matches('*');
        if name == asset && hash.len() == 64 {
            return Some(hash.to_ascii_lowercase());
        }
    }
    None
}

/// Extrai o binário `openmtr`/`openmtr.exe` de um arquivo `.tar.gz`, ignorando as demais entradas.
pub fn extract_tar_gz(bytes: &[u8]) -> Result<Vec<u8>, CliError> {
    let mut archive = tar::Archive::new(GzDecoder::new(bytes));
    let entries = archive
        .entries()
        .map_err(|e| CliError::Config(format!("ler arquivo: {e}")))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| CliError::Config(format!("ler entrada: {e}")))?;
        let path = entry
            .path()
            .map_err(|e| CliError::Config(format!("caminho inválido: {e}")))?
            .to_path_buf();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if name == "openmtr" || name == "openmtr.exe" {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| CliError::Config(format!("ler binário: {e}")))?;
            return Ok(buf);
        }
    }
    Err(CliError::Config(
        "binário openmtr não encontrado no arquivo".into(),
    ))
}
