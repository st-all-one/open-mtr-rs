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
use std::fmt;
use std::io::Read;

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
    /// Lê e valida o objeto de login.
    ///
    /// `-` lê o JSON do stdin; `@caminho` lê de um arquivo; qualquer outro
    /// valor é tratado como JSON inline.
    pub fn parse(raw: &str) -> Result<Self, CliError> {
        let text = if raw == "-" {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| CliError::Usage(format!("ler login do stdin: {e}")))?;
            buf
        } else if let Some(path) = raw.strip_prefix('@') {
            std::fs::read_to_string(path)
                .map_err(|e| CliError::Config(format!("ler login {path}: {e}")))?
        } else {
            raw.to_string()
        };
        let login: Self = serde_json::from_str(&text)
            .map_err(|e| CliError::Usage(format!("login inválido: {e}")))?;
        login.validate()?;
        Ok(login)
    }

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
                self.require_classic(true)?;
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
            (None, Some(name)) => Some(env_secret("token_env", &name)?),
            (None, None) => None,
        };
        let senha = match (self.senha, self.senha_env) {
            (Some(senha), _) => Some(senha),
            (None, Some(name)) => Some(env_secret("senha_env", &name)?),
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
            return self.require_classic(false);
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

    fn require_classic(&self, require_cnpj: bool) -> Result<(), CliError> {
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
        if require_cnpj
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

impl fmt::Debug for Login {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Login")
            .field("system", &self.system)
            .field("token", &self.token.as_ref().map(|_| "<redacted>"))
            .field("token_env", &self.token_env)
            .field("cpf", &self.cpf)
            .field("cnpj", &self.cnpj)
            .field("unidade", &self.unidade)
            .field("senha", &self.senha.as_ref().map(|_| "<redacted>"))
            .field("senha_env", &self.senha_env)
            .field("base_url", &self.base_url)
            .field("timeout_secs", &self.timeout_secs)
            .field("max_rps", &self.max_rps)
            .finish()
    }
}

fn env_secret(field: &str, name: &str) -> Result<String, CliError> {
    let value = std::env::var(name)
        .map_err(|_| CliError::Config(format!("login: variável {name} ({field}) não definida")))?;
    if value.trim().is_empty() {
        return Err(CliError::Config(format!(
            "login: variável {name} ({field}) vazia"
        )));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SINIR: &str = r#"{"system":"sinir","token":"super-secret"}"#;
    const MTRPE: &str = r#"{
        "system":"mtr_pe",
        "cnpj":"11222333000181",
        "cpf":"07800211428",
        "unidade":3615,
        "senha":"pw"
    }"#;

    #[test]
    fn sinir_accepts_token() {
        let login = Login::parse(SINIR).expect("login");
        let account = login.into_account().expect("conta");
        assert_eq!(account.system, System::Sinir);
        assert_eq!(account.token.as_deref(), Some("super-secret"));
        assert_eq!(account.cpf, None);
    }

    #[test]
    fn sinir_rejects_classic_credentials() {
        let raw = r#"{"system":"sinir","token":"t","cpf":"1","unidade":1,"senha":"p"}"#;
        assert!(Login::parse(raw).is_err());
    }

    #[test]
    fn sinir_requires_token() {
        assert!(Login::parse(r#"{"system":"sinir"}"#).is_err());
    }

    #[test]
    fn mtrpe_accepts_complete_login() {
        let login = Login::parse(MTRPE).expect("login");
        let account = login.into_account().expect("conta");
        assert_eq!(account.system, System::MtrPe);
        assert_eq!(account.cnpj.as_deref(), Some("11222333000181"));
        assert_eq!(account.unidade, Some(3615));
        assert_eq!(account.senha.as_deref(), Some("pw"));
        assert_eq!(account.token, None);
    }

    #[test]
    fn mtrpe_requires_each_field() {
        for raw in [
            r#"{"system":"mtr_pe","cpf":"1","unidade":1,"senha":"p"}"#,
            r#"{"system":"mtr_pe","cnpj":"1","unidade":1,"senha":"p"}"#,
            r#"{"system":"mtr_pe","cnpj":"1","cpf":"1","senha":"p"}"#,
            r#"{"system":"mtr_pe","cnpj":"1","cpf":"1","unidade":1}"#,
        ] {
            assert!(
                Login::parse(raw).is_err(),
                "aceitou login incompleto: {raw}"
            );
        }
    }

    #[test]
    fn mtrpe_rejects_token() {
        assert!(Login::parse(r#"{"system":"mtr_pe","token":"t"}"#).is_err());
    }

    #[test]
    fn sigor_accepts_token_or_classic() {
        let token = Login::parse(r#"{"system":"sigor","token":"t"}"#).expect("token");
        assert_eq!(token.into_account().expect("conta").system, System::Sigor);
        let classic = Login::parse(r#"{"system":"sigor","cpf":"1","unidade":2,"senha":"p"}"#)
            .expect("clássico");
        let account = classic.into_account().expect("conta");
        assert_eq!(account.cnpj, None);
    }

    #[test]
    fn sigor_rejects_mixed_auth() {
        let raw = r#"{"system":"sigor","token":"t","cpf":"1","unidade":2,"senha":"p"}"#;
        assert!(Login::parse(raw).is_err());
    }

    #[test]
    fn unknown_or_adapterless_system_is_rejected() {
        assert!(Login::parse(r#"{"system":"desconhecido","token":"t"}"#).is_err());
        assert!(Login::parse(r#"{"system":"mtr_ce","token":"t"}"#).is_err());
    }

    #[test]
    fn token_and_env_are_exclusive() {
        let raw = r#"{"system":"sinir","token":"t","token_env":"T"}"#;
        assert!(Login::parse(raw).is_err());
    }

    #[test]
    fn debug_redacts_secrets() {
        let login = Login::parse(SINIR).expect("login");
        let text = format!("{login:?}");
        assert!(!text.contains("super-secret"), "vazou token: {text}");
        assert!(text.contains("<redacted>"));
    }

    #[test]
    fn reads_login_from_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("login.json");
        std::fs::write(&path, SINIR).expect("write");
        Login::parse(&format!("@{}", path.display())).expect("login");
    }

    #[test]
    fn missing_env_is_an_error() {
        let login = Login::parse(r#"{"system":"sinir","token_env":"OPENMTR_MISSING_VAR_9E3A1C"}"#)
            .expect("login");
        assert!(login.into_account().is_err());
    }

    #[test]
    fn rejects_blank_token_and_senha() {
        assert!(Login::parse(r#"{"system":"sinir","token":"   "}"#).is_err());
        assert!(Login::parse(r#"{"system":"sinir","token_env":"  "}"#).is_err());
        assert!(
            Login::parse(r#"{"system":"mtr_pe","cnpj":"1","cpf":"1","unidade":1,"senha":""}"#)
                .is_err()
        );
        assert!(
            Login::parse(r#"{"system":"sigor","cpf":"1","unidade":1,"senha_env":" "}"#).is_err()
        );
    }

    #[test]
    fn invalid_login_error_does_not_echo_secret() {
        let err = Login::parse(r#"{"system":"sinir","token":"super-secret",}"#)
            .expect_err("json inválido");
        assert!(!err.to_string().contains("super-secret"));
    }

    #[test]
    fn into_account_applies_limits() {
        let raw = r#"{
            "system":"sinir","token":"t","timeout_secs":7,"max_rps":3,"base_url":"http://x"
        }"#;
        let account = Login::parse(raw)
            .expect("login")
            .into_account()
            .expect("conta");
        assert_eq!(account.max_concurrency, Some(3));
        assert_eq!(account.base_url.as_deref(), Some("http://x"));
    }
}
