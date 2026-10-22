use crate::config::{CliConfig, RawAccount};
use crate::error::CliError;
use open_mtr_core::openmtr::open_mtr_auth::{AuthParams, OpenMtrClassicAuth};
use open_mtr_core::openmtr::open_mtr_client::OpenMtrClient;
use open_mtr_core::openmtr::system::System;
// Arc removed

#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub label: Option<String>,
    pub system: System,
    pub tags: Vec<String>,
    pub access_type: String,
    pub access_priority: u8,
    pub base_url: Option<String>,
    pub token: Option<String>,
    pub token_env: Option<String>,
    pub cpf: Option<String>,
    pub senha: Option<String>,
    pub senha_env: Option<String>,
    pub unidade: Option<u32>,
    pub cnpj: Option<String>,
    pub max_concurrency: Option<u32>,
    pub enabled: bool,
}

impl Account {
    pub fn from_raw(r: &RawAccount) -> Result<Self, CliError> {
        let system = r.system.parse::<System>().map_err(CliError::Config)?;
        let at = r
            .access_type
            .clone()
            .unwrap_or_else(|| "geradora".to_string());
        let prio = match at.as_str() {
            "transportadora" => 0,
            "destinadora" | "receptora" => 1,
            "geradora" => 2,
            _ => return Err(CliError::Config(format!("access_type inválido: {at}"))),
        };
        Ok(Self {
            id: r.id.clone(),
            label: r.label.clone(),
            system,
            tags: r.tags.clone(),
            access_type: at,
            access_priority: prio,
            base_url: r.base_url.clone(),
            token: r.token.clone(),
            token_env: r.token_env.clone(),
            cpf: r.cpf.clone(),
            senha: r.senha.clone(),
            senha_env: r.senha_env.clone(),
            unidade: r.unidade,
            cnpj: r.cnpj.clone(),
            max_concurrency: r.max_concurrency,
            enabled: r.enabled.unwrap_or(true),
        })
    }

    pub fn system_slug(&self) -> &'static str {
        match self.system {
            open_mtr_core::openmtr::system::System::Sinir => "sinir",
            open_mtr_core::openmtr::system::System::Sigor => "sigor",
            open_mtr_core::openmtr::system::System::MtrPe => "mtr_pe",
            _ => "outro",
        }
    }

    pub fn resolve_token(&self) -> Result<Option<String>, CliError> {
        if let Some(t) = &self.token {
            return Ok(Some(t.clone()));
        }
        if let Some(env) = &self.token_env {
            return Ok(std::env::var(env).ok());
        }
        Ok(None)
    }

    pub fn resolve_senha(&self) -> Result<Option<String>, CliError> {
        if let Some(s) = &self.senha {
            return Ok(Some(s.clone()));
        }
        if let Some(env) = &self.senha_env {
            return Ok(std::env::var(env).ok());
        }
        Ok(None)
    }

    pub async fn build_client(
        &self,
        cfg: &CliConfig,
        fixed_time: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<OpenMtrClient, CliError> {
        let base = self.base_url.clone();
        let timeout = cfg.defaults.timeout_secs;
        let maxrps = self.max_concurrency.unwrap_or(cfg.defaults.max_concurrency);
        // auth
        let token = match self.resolve_token()? {
            Some(t) => t,
            None => {
                // classic auth
                let cpf = self
                    .cpf
                    .clone()
                    .ok_or_else(|| CliError::Auth("cpf ausente".into()))?;
                let senha = self
                    .resolve_senha()?
                    .ok_or_else(|| CliError::Auth("senha ausente".into()))?;
                let unidade = self
                    .unidade
                    .ok_or_else(|| CliError::Auth("unidade ausente".into()))?;
                let params = AuthParams::CpfSenhaUnidade {
                    cpf,
                    senha,
                    unidade,
                    cnpj: self.cnpj.clone(),
                };
                OpenMtrClassicAuth::new(self.system)
                    .base_url_opt(base.clone())
                    .timeout_secs(timeout)
                    .params(params)
                    .build()
                    .await
                    .map_err(|e| CliError::Auth(e.to_string()))?
            }
        };
        let mut b = OpenMtrClient::builder()
            .base(self.system)
            .token(&token)
            .timeout_secs(timeout)
            .max_rps(maxrps);
        if let Some(bu) = &base {
            b = b.base_url(bu.clone());
        }
        if let Some(ts) = fixed_time {
            b = b
                .clock(std::sync::Arc::new(
                    open_mtr_core::openmtr::time_util::FixedClock::new(ts),
                ))
                .id_generator(std::sync::Arc::new(
                    open_mtr_core::openmtr::time_util::SequentialIdGenerator::default(),
                ));
        }
        b.build().await.map_err(|e| CliError::Auth(e.to_string()))
    }
}

trait BaseUrlOpt {
    fn base_url_opt(self, url: Option<String>) -> Self;
}
impl BaseUrlOpt for OpenMtrClassicAuth {
    fn base_url_opt(self, url: Option<String>) -> Self {
        match url {
            Some(u) => self.base_url(u),
            None => self,
        }
    }
}

pub fn load_accounts(cfg: &CliConfig) -> Result<Vec<Account>, CliError> {
    cfg.accounts.iter().map(Account::from_raw).collect()
}

pub fn filter_accounts<'a>(
    accounts: &'a [Account],
    account_ids: &[String],
    system: Option<&str>,
    tags: &[String],
    all: bool,
) -> Result<Vec<&'a Account>, CliError> {
    let enabled: Vec<&Account> = accounts.iter().filter(|a| a.enabled).collect();
    if all {
        return Ok(enabled);
    }
    if !account_ids.is_empty() {
        let mut out = Vec::new();
        for id in account_ids {
            let a = enabled
                .iter()
                .find(|x| x.id == *id)
                .ok_or_else(|| CliError::NotFound(format!("conta não encontrada: {id}")))?;
            out.push(*a);
        }
        return Ok(out);
    }
    if let Some(sys) = system {
        let s: System = sys.parse().map_err(CliError::Config)?;
        return Ok(enabled.into_iter().filter(|a| a.system == s).collect());
    }
    if !tags.is_empty() {
        return Ok(enabled
            .into_iter()
            .filter(|a| tags.iter().all(|t| a.tags.contains(t)))
            .collect());
    }
    Ok(Vec::new())
}
