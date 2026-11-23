//! Atualização automática do binário a partir de releases do GitHub.
//!
//! O fluxo consulta a última tag (ou `--version`), monta o nome do asset para
//! o alvo `arch-os`, baixa `sha256sums.txt` e o arquivo, compara o SHA-256 e
//! só então substitui o executável de forma atômica (staging + `rename`,
//! modo `0755` em Unix). Bases alternativas podem vir de `OPENMTR_API_URL`,
//! `OPENMTR_RELEASES_URL`, `OPENMTR_VERSION` e `OPENMTR_INSTALL_PATH`, ou
//! serem sobrescritas em [`UpdateOptions`](crate::selfupdate::UpdateOptions).
use crate::error::CliError;
use serde::Serialize;
use std::path::PathBuf;

mod install;
mod release;

#[cfg(test)]
mod tests;

pub use install::{extract_tar_gz, parse_checksums};
pub use release::{asset_name, current_version, target};

use install::install;
use release::{download_base, latest_tag, normalize, release_tag};

/// Repositório GitHub de onde as releases são baixadas.
pub const REPO: &str = "st-all-one/open-mtr-rs";

/// Opções que controlam consulta, download e instalação da atualização.
#[allow(
    clippy::struct_excessive_bools,
    reason = "flags independentes da CLI: check, dry_run e force"
)]
#[derive(Debug, Clone, Default)]
pub struct UpdateOptions {
    /// Apenas consulta a release e monta o plano, sem instalar.
    pub check: bool,
    /// Baixa e valida o asset, mas não substitui o binário.
    pub dry_run: bool,
    /// Versão explícita (tag) a instalar; sobrepõe a última release.
    pub version: Option<String>,
    /// Instala mesmo quando a versão é igual à atual.
    pub force: bool,
    /// Base da API GitHub, substituindo `OPENMTR_API_URL`.
    pub api_base: Option<String>,
    /// Base de download das releases, substituindo `OPENMTR_RELEASES_URL`.
    pub download_base: Option<String>,
    /// Caminho de instalação, substituindo `OPENMTR_INSTALL_PATH`.
    pub install_path: Option<PathBuf>,
}

/// Plano de atualização calculado antes de qualquer modificação no sistema.
#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    /// Versão do binário em execução.
    pub current: String,
    /// Tag da release escolhida (sempre prefixada com `v`).
    pub latest: String,
    /// Tripla alvo no formato `arch-suffix` usada no nome do asset.
    pub target: String,
    /// Nome do asset baixado (`openmtr-<tag>-<alvo>.tar.gz` ou `.zip`).
    pub asset: String,
    /// URL de download do asset.
    pub asset_url: String,
    /// URL de download do `sha256sums.txt`.
    pub checksums_url: String,
    /// `true` quando a versão difere da atual.
    pub changed: bool,
    /// Ação derivada: `install` se houver mudança, `noop` caso contrário.
    pub action: String,
}

/// Consulta a release e monta um [`Plan`] sem baixar ou instalar nada.
pub async fn plan(opts: &UpdateOptions) -> Result<Plan, CliError> {
    let target = target()?;
    let client = reqwest::Client::new();
    let tag = release_tag(&match &opts.version {
        Some(v) => v.clone(),
        None => latest_tag(&client, opts).await?,
    });
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

/// Executa o plano; instala somente se houver mudança ou `force`, e nem `check` nem `dry_run` estiverem ativos.
pub async fn run(opts: &UpdateOptions) -> Result<Plan, CliError> {
    let plan = plan(opts).await?;
    let should_install = plan.changed || opts.force;
    if opts.check || opts.dry_run || !should_install {
        return Ok(plan);
    }
    install(&plan, opts).await?;
    Ok(plan)
}
