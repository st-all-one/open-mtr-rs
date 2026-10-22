use crate::error::CliError;
use flate2::read::GzDecoder;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};

pub const REPO: &str = "st-all-one/open-mtr-rs";
const DEFAULT_API: &str = "https://api.github.com";
const DEFAULT_DOWNLOAD: &str = "https://github.com";

#[derive(Debug, Clone, Default)]
pub struct UpdateOptions {
    pub check: bool,
    pub dry_run: bool,
    pub version: Option<String>,
    pub force: bool,
    pub api_base: Option<String>,
    pub download_base: Option<String>,
    pub install_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    pub current: String,
    pub latest: String,
    pub target: String,
    pub asset: String,
    pub asset_url: String,
    pub checksums_url: String,
    pub changed: bool,
    pub action: String,
}

pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn target() -> Result<String, CliError> {
    let arch = std::env::consts::ARCH;
    let arch = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => {
            return Err(CliError::Usage(format!(
                "self update: arquitetura não suportada: {other}"
            )));
        }
    };
    let os = std::env::consts::OS;
    let suffix = match os {
        "linux" => "unknown-linux-musl",
        "macos" => "apple-darwin",
        "windows" => "pc-windows-msvc",
        other => {
            return Err(CliError::Usage(format!(
                "self update: sistema não suportado: {other}"
            )));
        }
    };
    Ok(format!("{arch}-{suffix}"))
}

pub fn asset_name(tag: &str, target: &str) -> String {
    if target.contains("windows") {
        format!("openmtr-{tag}-{target}.zip")
    } else {
        format!("openmtr-{tag}-{target}.tar.gz")
    }
}

fn api_base(opts: &UpdateOptions) -> String {
    opts.api_base
        .clone()
        .or_else(|| std::env::var("OPENMTR_API_URL").ok())
        .unwrap_or_else(|| DEFAULT_API.to_string())
}

fn download_base(opts: &UpdateOptions) -> String {
    opts.download_base
        .clone()
        .or_else(|| std::env::var("OPENMTR_RELEASES_URL").ok())
        .unwrap_or_else(|| DEFAULT_DOWNLOAD.to_string())
}

async fn latest_tag(client: &reqwest::Client, opts: &UpdateOptions) -> Result<String, CliError> {
    if let Ok(tag) = std::env::var("OPENMTR_VERSION") {
        return Ok(tag);
    }
    let url = format!("{}/repos/{REPO}/releases/latest", api_base(opts));
    let resp = client
        .get(&url)
        .header("User-Agent", concat!("openmtr/", env!("CARGO_PKG_VERSION")))
        .send()
        .await
        .map_err(|e| CliError::Network(format!("consultar release: {e}")))?;
    if !resp.status().is_success() {
        return Err(CliError::Network(format!(
            "consultar release: HTTP {}",
            resp.status()
        )));
    }
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| CliError::Network(format!("release inválida: {e}")))?;
    body.get("tag_name")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| CliError::Network("release sem tag_name".into()))
}

pub async fn plan(opts: &UpdateOptions) -> Result<Plan, CliError> {
    let target = target()?;
    let client = reqwest::Client::new();
    let tag = match &opts.version {
        Some(v) => v.clone(),
        None => latest_tag(&client, opts).await?,
    };
    let asset = asset_name(&tag, &target);
    let base = download_base(opts);
    let asset_url = format!("{base}/{REPO}/releases/download/{tag}/{asset}");
    let checksums_url = format!("{base}/{REPO}/releases/download/{tag}/sha256sums.txt");
    let current = current_version().to_string();
    let changed = normalize(&tag) != normalize(&current);
    let action = if changed { "install" } else { "noop" };
    Ok(Plan {
        current,
        latest: tag,
        target,
        asset,
        asset_url,
        checksums_url,
        changed,
        action: action.to_string(),
    })
}

pub async fn run(opts: &UpdateOptions) -> Result<Plan, CliError> {
    let plan = plan(opts).await?;
    if opts.check || opts.dry_run || (!plan.changed && !opts.force) {
        return Ok(plan);
    }
    install(&plan, opts).await?;
    Ok(plan)
}

async fn install(plan: &Plan, opts: &UpdateOptions) -> Result<(), CliError> {
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
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "openmtr".into()),
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
        let _ = std::fs::remove_file(&staging);
        CliError::Config(format!("instalar binário: {e}"))
    })
}

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

fn normalize(v: &str) -> String {
    v.trim().trim_start_matches('v').to_string()
}
