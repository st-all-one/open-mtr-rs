use crate::error::CliError;
use open_mtr_core::openmtr::system::System;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAccount {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    pub system: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub access_type: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub token_env: Option<String>,
    #[serde(default)]
    pub cpf: Option<String>,
    #[serde(default)]
    pub senha: Option<String>,
    #[serde(default)]
    pub senha_env: Option<String>,
    #[serde(default)]
    pub unidade: Option<u32>,
    #[serde(default)]
    pub cnpj: Option<String>,
    #[serde(default)]
    pub max_concurrency: Option<u32>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
    #[serde(default = "d_window")]
    pub dump_window_days: u32,
    #[serde(default = "d_conc")]
    pub max_concurrency: u32,
    #[serde(default = "d_timeout")]
    pub timeout_secs: u64,
}

fn d_window() -> u32 {
    30
}
fn d_conc() -> u32 {
    10
}
fn d_timeout() -> u64 {
    30
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            dump_window_days: 30,
            max_concurrency: 10,
            timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliConfig {
    #[serde(default = "d_cv")]
    pub config_version: u32,
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub accounts: Vec<RawAccount>,
}

fn d_cv() -> u32 {
    1
}

impl CliConfig {
    pub fn load(path: &Path) -> Result<Self, CliError> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            CliError::Config(format!("não foi possível ler {}: {e}", path.display()))
        })?;
        let cfg: CliConfig =
            toml::from_str(&text).map_err(|e| CliError::Config(format!("TOML inválido: {e}")))?;
        cfg.validate()?;
        warn_inline_secrets(path, &cfg);
        Ok(cfg)
    }

    #[allow(
        clippy::excessive_nesting,
        reason = "validação linear de contas; extrair função prejudicaria a leitura"
    )]
    pub fn validate(&self) -> Result<(), CliError> {
        if self.config_version != 1 {
            return Err(CliError::Config(format!(
                "config_version não suportada: {} (suportada: 1)",
                self.config_version
            )));
        }
        let mut ids = std::collections::HashSet::new();
        for a in &self.accounts {
            if a.id.trim().is_empty() {
                return Err(CliError::Config("id de conta vazio".into()));
            }
            if !ids.insert(a.id.clone()) {
                return Err(CliError::Config(format!("id duplicado: {}", a.id)));
            }
            let _ = a
                .system
                .parse::<System>()
                .map_err(|_| CliError::Config(format!("sistema inválido: {}", a.system)))?;
            if let Some(at) = &a.access_type {
                match at.as_str() {
                    "transportadora" | "destinadora" | "receptora" | "geradora" => {}
                    other => {
                        return Err(CliError::Config(format!(
                            "conta {}: access_type inválido: {other}",
                            a.id
                        )));
                    }
                }
            }
            validate_auth(a)?;
        }
        Ok(())
    }
}

fn validate_auth(a: &RawAccount) -> Result<(), CliError> {
    if a.token.is_some() && a.token_env.is_some() {
        return Err(CliError::Config(format!(
            "conta {}: use token ou token_env, não ambos",
            a.id
        )));
    }
    if a.senha.is_some() && a.senha_env.is_some() {
        return Err(CliError::Config(format!(
            "conta {}: use senha ou senha_env, não ambos",
            a.id
        )));
    }
    let has_token = a.token.is_some() || a.token_env.is_some();
    let has_classic = a.cpf.is_some() || a.senha.is_some() || a.senha_env.is_some();
    if has_token && has_classic {
        return Err(CliError::Config(format!(
            "conta {}: escolha token ou credenciais clássicas, não ambos",
            a.id
        )));
    }
    if !has_token {
        if a.cpf.is_none() {
            return Err(CliError::Config(format!(
                "conta {}: defina token/token_env ou cpf+senha+unidade",
                a.id
            )));
        }
        if a.senha.is_none() && a.senha_env.is_none() {
            return Err(CliError::Config(format!(
                "conta {}: senha ou senha_env ausente",
                a.id
            )));
        }
        if a.unidade.is_none() {
            return Err(CliError::Config(format!("conta {}: unidade ausente", a.id)));
        }
    }
    if a.base_url.as_deref().is_some_and(|u| u.trim().is_empty()) {
        return Err(CliError::Config(format!("conta {}: base_url vazia", a.id)));
    }
    Ok(())
}

#[allow(
    clippy::excessive_nesting,
    reason = "checagem de permissão unix + aviso por conta; linear e curta"
)]
fn warn_inline_secrets(path: &Path, cfg: &CliConfig) {
    let inline: Vec<&RawAccount> = cfg
        .accounts
        .iter()
        .filter(|a| a.token.is_some() || a.senha.is_some())
        .collect();
    if inline.is_empty() {
        return;
    }
    for a in &inline {
        tracing::warn!(
            account = %a.id,
            config = %path.display(),
            "segredo inline; prefira token_env/senha_env"
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let mode = meta.permissions().mode();
            if mode & 0o077 != 0 {
                tracing::warn!(
                    config = %path.display(),
                    mode = format_args!("{:o}", mode & 0o777),
                    "config legível por grupo/outros; use chmod 600"
                );
            }
        }
    }
}

pub fn parse_system(s: &str) -> Result<System, CliError> {
    s.parse::<System>()
        .map_err(|_| CliError::Config(format!("sistema inválido: {s}")))
}
