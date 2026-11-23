//! Objeto de login explícito por sistema.
//!
//! O `--login` aceita **um único JSON** cujo esquema depende do sistema-alvo:
//!
//! - `sinir`: `{"system":"sinir","token":"..."}` (refresh token, ou `token_env`);
//! - `mtr_pe`: `{"system":"mtr_pe","cnpj":"...","cpf":"...","unidade":3615,"senha":"..."}`
//!   (ou `senha_env`);
//! - `sigor`: token **ou** credenciais clássicas (`cpf`, `unidade`, `senha`/`senha_env`).
//!
//! O mesmo objeto é aceito por todos os comandos e dispensa o
//! `open-mtr-config.toml`. Segredos nunca aparecem em [`Debug`].

use crate::account::Account;
use crate::error::CliError;
use open_mtr_core::openmtr::system::System;
use serde::Deserialize;

mod debug;
mod parse;
mod secrets;

#[cfg(test)]
mod tests;

/// Identificador reservado da conta efêmera criada por `--login`.
pub const LOGIN_ID: &str = "login";

/// Credenciais explícitas de uma sessão única.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Login {
    /// Sistema-alvo (`sinir`, `mtr_pe`, `sigor`).
    pub system: String,
    /// Refresh token do SINIR ou JWT direto do SIGOR/MTR-PE.
    #[serde(default)]
    pub token: Option<String>,
    /// Variável de ambiente com o token.
    #[serde(default)]
    pub token_env: Option<String>,
    /// CPF do usuário (MTR-PE e SIGOR clássico).
    #[serde(default)]
    pub cpf: Option<String>,
    /// CNPJ da empresa (obrigatório no MTR-PE).
    #[serde(default)]
    pub cnpj: Option<String>,
    /// Código da unidade (MTR-PE e SIGOR clássico).
    #[serde(default)]
    pub unidade: Option<u32>,
    /// Senha do usuário (MTR-PE e SIGOR clássico).
    #[serde(default)]
    pub senha: Option<String>,
    /// Variável de ambiente com a senha.
    #[serde(default)]
    pub senha_env: Option<String>,
    /// Override opcional da URL base do sistema.
    #[serde(default)]
    pub base_url: Option<String>,
    /// Timeout HTTP em segundos (1..=3600).
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    /// Limite de requisições por segundo (1..=32).
    #[serde(default)]
    pub max_rps: Option<u32>,
}

impl Login {
    /// Valida presença e exclusividade dos campos para o sistema-alvo.
    pub fn validate(&self) -> Result<(), CliError> {
        let system = self.parsed_system()?;
        if self.timeout_secs.is_some_and(|t| !(1..=3600).contains(&t)) {
            return Err(CliError::Usage(
                "login.timeout_secs deve estar entre 1 e 3600".into(),
            ));
        }
        if self.max_rps.is_some_and(|r| !(1..=32).contains(&r)) {
            return Err(CliError::Usage(
                "login.max_rps deve estar entre 1 e 32".into(),
            ));
        }
        if self
            .base_url
            .as_deref()
            .is_some_and(|url| url.trim().is_empty())
        {
            return Err(CliError::Usage("login.base_url vazia".into()));
        }
        if self.token.is_some() && self.token_env.is_some() {
            return Err(CliError::Usage(
                "login: use token ou token_env, não ambos".into(),
            ));
        }
        if self.senha.is_some() && self.senha_env.is_some() {
            return Err(CliError::Usage(
                "login: use senha ou senha_env, não ambos".into(),
            ));
        }
        if self.token.as_deref().is_some_and(|v| v.trim().is_empty())
            || self
                .token_env
                .as_deref()
                .is_some_and(|v| v.trim().is_empty())
        {
            return Err(CliError::Usage("login: token/token_env vazio".into()));
        }
        if self.senha.as_deref().is_some_and(|v| v.trim().is_empty())
            || self
                .senha_env
                .as_deref()
                .is_some_and(|v| v.trim().is_empty())
        {
            return Err(CliError::Usage("login: senha/senha_env vazia".into()));
        }
        match system {
            System::Sinir => {
                self.require_token()?;
                self.reject_classic("sinir")?;
            }
            System::MtrPe => {
                self.reject_token("mtr_pe")?;
                self.require_classic(CnpjRequirement::Required)?;
            }
            System::Sigor => self.validate_sigor()?,
            other => {
                return Err(CliError::Usage(format!(
                    "login: sistema sem adapter: {other}"
                )));
            }
        }
        Ok(())
    }

    /// Converte o login em uma conta efêmera pronta para [`Account::build_client`].
    ///
    /// Resolve `token_env`/`senha_env` agora; a conta resultante guarda apenas
    /// os valores resolvidos.
    pub fn into_account(self) -> Result<Account, CliError> {
        let system = self.parsed_system()?;
        let token = match (self.token, self.token_env) {
            (Some(token), _) => Some(token),
            (None, Some(name)) => Some(secrets::env_secret("token_env", &name)?),
            (None, None) => None,
        };
        let senha = match (self.senha, self.senha_env) {
            (Some(senha), _) => Some(senha),
            (None, Some(name)) => Some(secrets::env_secret("senha_env", &name)?),
            (None, None) => None,
        };
        Ok(Account {
            id: LOGIN_ID.to_string(),
            label: Some("login efêmero".to_string()),
            system,
            tags: Vec::new(),
            access_type: "geradora".to_string(),
            access_priority: 2,
            base_url: self.base_url,
            token,
            token_env: None,
            cpf: self.cpf,
            senha,
            senha_env: None,
            unidade: self.unidade,
            cnpj: self.cnpj,
            max_concurrency: self.max_rps,
            enabled: true,
        })
    }

    fn parsed_system(&self) -> Result<System, CliError> {
        self.system
            .parse::<System>()
            .map_err(|_| CliError::Usage(format!("login.system inválido: {}", self.system)))
    }

    fn has_token(&self) -> bool {
        self.token.is_some() || self.token_env.is_some()
    }

    fn has_senha(&self) -> bool {
        self.senha.is_some() || self.senha_env.is_some()
    }

    fn has_classic(&self) -> bool {
        self.cpf.is_some() || self.cnpj.is_some() || self.unidade.is_some() || self.has_senha()
    }

    fn validate_sigor(&self) -> Result<(), CliError> {
        if !self.has_token() {
            return self.require_classic(CnpjRequirement::Optional);
        }
        self.reject_classic("sigor")?;
        self.require_token()
    }

    fn reject_classic(&self, system: &str) -> Result<(), CliError> {
        if self.has_classic() {
            return Err(CliError::Usage(format!(
                "login {system}: use token ou credenciais clássicas, não ambos"
            )));
        }
        Ok(())
    }

    fn reject_token(&self, system: &str) -> Result<(), CliError> {
        if self.has_token() {
            return Err(CliError::Usage(format!(
                "login {system}: use credenciais clássicas, não token"
            )));
        }
        Ok(())
    }

    fn require_token(&self) -> Result<(), CliError> {
        if self.has_token() {
            Ok(())
        } else {
            Err(CliError::Usage(format!(
                "login {} exige token ou token_env",
                self.system
            )))
        }
    }

    fn require_classic(&self, cnpj: CnpjRequirement) -> Result<(), CliError> {
        let mut missing = Vec::new();
        if self
            .cpf
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            missing.push("cpf");
        }
        if self.unidade.is_none() {
            missing.push("unidade");
        }
        if !self.has_senha() {
            missing.push("senha/senha_env");
        }
        if matches!(cnpj, CnpjRequirement::Required)
            && self
                .cnpj
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
        {
            missing.push("cnpj");
        }
        if missing.is_empty() {
            Ok(())
        } else {
            Err(CliError::Usage(format!(
                "login {} exige {}",
                self.system,
                missing.join(", ")
            )))
        }
    }
}

/// Define se o CNPJ é obrigatório nas credenciais clássicas.
#[derive(Clone, Copy, Debug)]
enum CnpjRequirement {
    /// CNPJ obrigatório (MTR-PE).
    Required,
    /// CNPJ opcional (SIGOR).
    Optional,
}
