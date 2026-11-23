//! Leitura e validação do arquivo de configuração TOML da CLI.
//!
//! O arquivo tem `config_version = 1`, uma tabela `[defaults]` opcional e uma
//! lista de contas `[[accounts]]`. Cada conta identifica um `system`
//! (`sinir`, `sigor`, `mtr-pe`/`mtr_pe`/`mtrpe`, etc.), um `id` único, e
//! credenciais via token (`token`/`token_env`) ou via par CPF/senha/unidade
//! (`cpf`, `senha`/`senha_env`, `unidade`) — nunca ambos. `deny_unknown_fields`
//! rejeita chaves desconhecidas. Segredos inline geram aviso; prefira as
//! variantes `_env`, e em Unix o arquivo deve ter permissão `0600`.

use crate::error::CliError;
use open_mtr_core::openmtr::system::System;
use serde::Deserialize;
use std::path::Path;

/// Uma conta declarada em `[[accounts]]` no TOML de configuração.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAccount {
    /// Identificador único da conta; usado por `--account` e no estado de retomada.
    pub id: String,
    /// Rótulo legível da conta, opcional.
    #[serde(default)]
    pub label: Option<String>,
    /// Sistema de origem; aceita `sinir`, `sigor`, `mtr-pe`/`mtr_pe`/`mtrpe` e equivalentes estaduais.
    pub system: String,
    /// Etiquetas da conta para seleção por `--tag` (todas devem corresponder).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Papel da conta: `transportadora`, `destinadora`, `receptora` ou `geradora` (padrão).
    #[serde(default)]
    pub access_type: Option<String>,
    /// URL base da API, opcional; quando ausente usa a URL padrão do sistema.
    #[serde(default)]
    pub base_url: Option<String>,
    /// Token de acesso inline; exclusivo com `token_env` e com credenciais clássicas.
    #[serde(default)]
    pub token: Option<String>,
    /// Nome da variável de ambiente que contém o token; alternativa segura a `token`.
    #[serde(default)]
    pub token_env: Option<String>,
    /// CPF da autenticação clássica; obrigatório quando não há token.
    #[serde(default)]
    pub cpf: Option<String>,
    /// Senha inline da autenticação clássica; exclusiva com `senha_env`.
    #[serde(default)]
    pub senha: Option<String>,
    /// Nome da variável de ambiente que contém a senha clássica.
    #[serde(default)]
    pub senha_env: Option<String>,
    /// Unidade associada ao CPF na autenticação clássica; obrigatória sem token.
    #[serde(default)]
    pub unidade: Option<u32>,
    /// CNPJ opcional da autenticação clássica.
    #[serde(default)]
    pub cnpj: Option<String>,
    /// Limite de concorrência específico da conta; sobrepõe `defaults.max_concurrency`.
    #[serde(default)]
    pub max_concurrency: Option<u32>,
    /// Habilita a conta; padrão `true` quando omitido.
    #[serde(default)]
    pub enabled: Option<bool>,
}

/// Valores padrão da tabela `[defaults]`, aplicados às contas que não os definem.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
    /// Janela de dias usada pelo dump ao dividir o período (padrão `30`).
    #[serde(default = "d_window")]
    pub dump_window_days: u32,
    /// Concorrência máxima padrão (padrão `10`).
    #[serde(default = "d_conc")]
    pub max_concurrency: u32,
    /// Timeout das requisições HTTP em segundos (padrão `30`).
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

/// Configuração raiz do arquivo TOML.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliConfig {
    /// Versão do formato; apenas `1` é suportada por este binário.
    #[serde(default = "d_cv")]
    pub config_version: u32,
    /// Padrões globais de execução.
    #[serde(default)]
    pub defaults: Defaults,
    /// Contas disponíveis; a seleção de escopo filtra as habilitadas.
    #[serde(default)]
    pub accounts: Vec<RawAccount>,
}

fn d_cv() -> u32 {
    1
}

impl CliConfig {
    /// Lê `path`, analisa o TOML, valida o conteúdo e avisa sobre segredos inline.
    pub fn load(path: &Path) -> Result<Self, CliError> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            CliError::Config(format!("não foi possível ler {}: {e}", path.display()))
        })?;
        let cfg: Self =
            toml::from_str(&text).map_err(|e| CliError::Config(format!("TOML inválido: {e}")))?;
        cfg.validate()?;
        warn_inline_secrets(path, &cfg);
        Ok(cfg)
    }

    /// Valida versão, ids únicos, sistemas, `access_type` e a combinação de credenciais.
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

/// Converte uma string de sistema em [`System`], mapeando falha para [`CliError::Config`].
pub fn parse_system(s: &str) -> Result<System, CliError> {
    s.parse::<System>()
        .map_err(|_| CliError::Config(format!("sistema inválido: {s}")))
}
