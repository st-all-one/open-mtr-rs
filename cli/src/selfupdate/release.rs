use super::{REPO, UpdateOptions};
use crate::error::CliError;

const DEFAULT_API: &str = "https://api.github.com";
const DEFAULT_DOWNLOAD: &str = "https://github.com";

/// Versão do pacote em execução, extraída de `CARGO_PKG_VERSION` em tempo de compilação.
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Tripla alvo derivada de `ARCH`/`OS`; falha com `Usage` para arquiteturas ou sistemas não suportados.
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

/// Nome do asset de release: `.zip` no Windows, `.tar.gz` nos demais sistemas.
pub fn asset_name(tag: &str, target: &str) -> String {
    if target.contains("windows") {
        format!("openmtr-{tag}-{target}.zip")
    } else {
        format!("openmtr-{tag}-{target}.tar.gz")
    }
}

pub(super) fn release_tag(tag: &str) -> String {
    let tag = tag.trim();
    if tag.starts_with('v') {
        tag.to_string()
    } else {
        format!("v{tag}")
    }
}

pub(super) fn normalize(v: &str) -> String {
    v.trim().trim_start_matches('v').to_string()
}

fn api_base(opts: &UpdateOptions) -> String {
    opts.api_base
        .clone()
        .or_else(|| std::env::var("OPENMTR_API_URL").ok())
        .unwrap_or_else(|| DEFAULT_API.to_string())
}

pub(super) fn download_base(opts: &UpdateOptions) -> String {
    opts.download_base
        .clone()
        .or_else(|| std::env::var("OPENMTR_RELEASES_URL").ok())
        .unwrap_or_else(|| DEFAULT_DOWNLOAD.to_string())
}

pub(super) async fn latest_tag(
    client: &reqwest::Client,
    opts: &UpdateOptions,
) -> Result<String, CliError> {
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
