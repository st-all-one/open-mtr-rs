//! Carregamento e autenticação de contas configuradas.
//!
//! Uma [`Account`](crate::account::Account) é a forma resolvida de um `RawAccount`: o `access_type`
//! vira `access_priority` para desempate de deduplicação e as credenciais
//! inline ou por variável de ambiente alimentam a construção do cliente.
//! O cache de sessão reaproveita access tokens e evita repetir login/refresh
//! em cada operação de um batch.
use crate::config::{CliConfig, RawAccount};
use crate::error::CliError;
use crate::session::SessionCache;
use open_mtr_core::openmtr::open_mtr_auth::{AuthParams, OpenMtrClassicAuth};
use open_mtr_core::openmtr::open_mtr_client::OpenMtrClient;
use open_mtr_core::openmtr::system::System;
// Arc removed
mod scope;

pub use scope::{AccountScope, filter_accounts};

/// Conta resolvida a partir da configuração, pronta para construir clientes e selecionar escopos.
#[derive(Debug, Clone)]
pub struct Account {
    /// Identificador único da conta.
    pub id: String,
    /// Rótulo legível, opcional.
    pub label: Option<String>,
    /// Sistema de origem dos manifestos.
    pub system: System,
    /// Etiquetas para seleção por `--tag`.
    pub tags: Vec<String>,
    /// Papel textual da conta (`transportadora`, `destinadora`, `receptora` ou `geradora`).
    pub access_type: String,
    /// Prioridade na deduplicação: transportadora `0`, destinadora/receptora `1`, geradora `2`.
    pub access_priority: u8,
    /// URL base da API, opcional.
    pub base_url: Option<String>,
    /// Token inline, usado diretamente em MTR-PE/SIGOR ou como refresh token no SINIR.
    pub token: Option<String>,
    /// Nome da variável de ambiente do token.
    pub token_env: Option<String>,
    /// CPF da autenticação clássica.
    pub cpf: Option<String>,
    /// Senha inline da autenticação clássica.
    pub senha: Option<String>,
    /// Nome da variável de ambiente da senha clássica.
    pub senha_env: Option<String>,
    /// Unidade da autenticação clássica.
    pub unidade: Option<u32>,
    /// CNPJ opcional da autenticação clássica.
    pub cnpj: Option<String>,
    /// Concorrência específica da conta; sobrepõe o padrão da configuração.
    pub max_concurrency: Option<u32>,
    /// Se a conta participa da seleção de escopo.
    pub enabled: bool,
}

impl Account {
    /// Converte um [`crate::config::RawAccount`] em conta resolvida, validando sistema e `access_type`.
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

    /// Slug estável do sistema usado como parte da chave do cache de sessão (`sinir`, `sigor`, `mtr_pe`, `outro`).
    pub fn system_slug(&self) -> &'static str {
        match self.system {
            System::Sinir => "sinir",
            System::Sigor => "sigor",
            System::MtrPe => "mtr_pe",
            _ => "outro",
        }
    }

    /// Retorna o token inline ou, na ausência dele, o valor da variável `token_env`.
    pub fn resolve_token(&self) -> Option<String> {
        self.token.clone().or_else(|| {
            self.token_env
                .as_ref()
                .and_then(|env| std::env::var(env).ok())
        })
    }

    /// Retorna a senha inline ou, na ausência dela, o valor da variável `senha_env`.
    pub fn resolve_senha(&self) -> Option<String> {
        self.senha.clone().or_else(|| {
            self.senha_env
                .as_ref()
                .and_then(|env| std::env::var(env).ok())
        })
    }

    /// Constrói o cliente reaproveitando um access token curto, se houver.
    ///
    /// O cache evita repetir `POST /token` (SINIR) ou login clássico
    /// (MTR-PE/SIGOR) a cada operação de um batch. Tokens diretos
    /// (`token`/`token_env` em MTR-PE/SIGOR) não passam pelo cache.
    pub async fn build_client(
        &self,
        cfg: &CliConfig,
        fixed_time: Option<jiff::Timestamp>,
        cache: &SessionCache,
    ) -> Result<OpenMtrClient, CliError> {
        let token = self.resolve_token();
        let uses_refresh = matches!(self.system, System::Sinir);
        let uses_classic = token.is_none() && !uses_refresh;

        if (uses_refresh || uses_classic)
            && let Some(cached) = cache.get(self)
        {
            return self.client_with_access(cfg, fixed_time, &cached).await;
        }

        if uses_refresh {
            return self
                .build_refresh_client(cfg, fixed_time, token, cache)
                .await;
        }

        let token = match token {
            Some(token) => token,
            None => self.classic_token(cfg).await?,
        };
        let client = self.client_with_access(cfg, fixed_time, &token).await?;
        if uses_classic {
            cache.put(self, &token);
        }
        Ok(client)
    }

    async fn build_refresh_client(
        &self,
        cfg: &CliConfig,
        fixed_time: Option<jiff::Timestamp>,
        token: Option<String>,
        cache: &SessionCache,
    ) -> Result<OpenMtrClient, CliError> {
        let refresh = token.ok_or_else(|| CliError::Auth("token de refresh ausente".into()))?;
        let client = self.client_with_refresh(cfg, fixed_time, &refresh).await?;
        if let Ok(access) = client.access_token().await {
            cache.put(self, &access);
        }
        Ok(client)
    }

    async fn classic_token(&self, cfg: &CliConfig) -> Result<String, CliError> {
        let cpf = self
            .cpf
            .clone()
            .ok_or_else(|| CliError::Auth("cpf ausente".into()))?;
        let senha = self
            .resolve_senha()
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
            .base_url_opt(self.base_url.clone())
            .timeout_secs(cfg.defaults.timeout_secs)
            .params(params)
            .build()
            .await
            .map_err(|e| CliError::Auth(e.to_string()))
    }

    fn configured_builder(
        &self,
        cfg: &CliConfig,
        fixed_time: Option<jiff::Timestamp>,
    ) -> open_mtr_core::openmtr::open_mtr_client::OpenMtrClientBuilder {
        let timeout = cfg.defaults.timeout_secs;
        let maxrps = self.max_concurrency.unwrap_or(cfg.defaults.max_concurrency);
        let mut builder = OpenMtrClient::builder()
            .base(self.system)
            .timeout_secs(timeout)
            .max_rps(maxrps);
        if let Some(base) = &self.base_url {
            builder = builder.base_url(base.clone());
        }
        if let Some(ts) = fixed_time {
            builder = builder
                .clock(std::sync::Arc::new(
                    open_mtr_core::openmtr::time_util::FixedClock::new(ts),
                ))
                .id_generator(std::sync::Arc::new(
                    open_mtr_core::openmtr::time_util::SequentialIdGenerator::default(),
                ));
        }
        builder
    }

    async fn client_with_access(
        &self,
        cfg: &CliConfig,
        fixed_time: Option<jiff::Timestamp>,
        token: &str,
    ) -> Result<OpenMtrClient, CliError> {
        self.configured_builder(cfg, fixed_time)
            .access_token(token)
            .build()
            .await
            .map_err(|e| CliError::Auth(e.to_string()))
    }

    async fn client_with_refresh(
        &self,
        cfg: &CliConfig,
        fixed_time: Option<jiff::Timestamp>,
        token: &str,
    ) -> Result<OpenMtrClient, CliError> {
        self.configured_builder(cfg, fixed_time)
            .token(token)
            .build()
            .await
            .map_err(|e| CliError::Auth(e.to_string()))
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

/// Converte todas as contas da configuração; falha na primeira conta inválida.
pub fn load_accounts(cfg: &CliConfig) -> Result<Vec<Account>, CliError> {
    cfg.accounts.iter().map(Account::from_raw).collect()
}
