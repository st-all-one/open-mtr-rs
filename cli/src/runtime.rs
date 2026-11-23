//! Estado compartilhado de uma execução da CLI: configuração, contas e cache de sessão.
use crate::account::{Account, AccountScope, filter_accounts, load_accounts};
use crate::config::{CliConfig, Defaults};
use crate::error::CliError;
use crate::login::{LOGIN_ID, Login};
use crate::session::SessionCache;
use std::path::Path;

/// Estado resolvido de uma invocação, carregado de `--config` ou de `--login`.
pub struct Runtime {
    /// Configuração carregada; vazia de contas no modo `--login`.
    pub config: CliConfig,
    /// Contas resolvidas e habilitadas para seleção.
    pub accounts: Vec<Account>,
    /// Cache de access tokens reaproveitado entre operações.
    pub cache: SessionCache,
}

impl Runtime {
    /// Carrega configuração e contas de `config_path` e cria o cache de sessão do ambiente.
    pub fn load(config_path: &Path) -> Result<Self, CliError> {
        let config = CliConfig::load(config_path)?;
        let accounts = load_accounts(&config)?;
        Ok(Self {
            config,
            accounts,
            cache: SessionCache::from_env(),
        })
    }

    /// Cria o runtime efêmero de uma sessão `--login` (sem config).
    pub fn from_login(login: Login) -> Result<Self, CliError> {
        login.validate()?;
        let timeout_secs = login.timeout_secs;
        let max_rps = login.max_rps;
        let account = login.into_account()?;
        let mut defaults = Defaults::default();
        if let Some(timeout) = timeout_secs {
            defaults.timeout_secs = timeout;
        }
        if let Some(rps) = max_rps {
            defaults.max_concurrency = rps;
        }
        Ok(Self {
            config: CliConfig {
                config_version: 1,
                defaults,
                accounts: Vec::new(),
            },
            accounts: vec![account],
            cache: SessionCache::from_env(),
        })
    }

    /// `true` quando o runtime veio de `--login` em vez de `--config`.
    pub fn is_login(&self) -> bool {
        self.config.accounts.is_empty()
            && matches!(self.accounts.as_slice(), [account] if account.id == LOGIN_ID)
    }

    /// Resolve o escopo de leitura, aceitando `--login` como conta única.
    ///
    /// Flags de escopo são incompatíveis com `--login`: a sessão já é única.
    pub fn select<'a>(&'a self, scope: AccountScope<'_>) -> Result<Vec<&'a Account>, CliError> {
        if !self.is_login() {
            return self.resolve(scope);
        }
        if !matches!(scope, AccountScope::All) {
            return Err(CliError::Usage(
                "--login não combina com --account/--system/--tag/--all-accounts".into(),
            ));
        }
        let account = self
            .accounts
            .first()
            .ok_or_else(|| CliError::Internal("runtime de login sem conta".into()))?;
        Ok(vec![account])
    }

    /// Conta única de uma operação de escrita.
    pub fn single(&self) -> Result<&Account, CliError> {
        if self.is_login() {
            self.accounts
                .first()
                .ok_or_else(|| CliError::Internal("runtime de login sem conta".into()))
        } else {
            Err(CliError::Usage(
                "informe --account <ID> ou --login <JSON|@ARQUIVO|->".into(),
            ))
        }
    }

    /// Filtra as contas pelo escopo sem tratar o modo `--login` como conta única.
    pub fn resolve<'a>(&'a self, scope: AccountScope<'_>) -> Result<Vec<&'a Account>, CliError> {
        filter_accounts(&self.accounts, scope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn login(raw: &str) -> Login {
        Login::parse(raw).expect("login válido")
    }

    #[test]
    fn login_runtime_applies_timeout_and_limits() {
        let rt = Runtime::from_login(login(
            r#"{"system":"sinir","token":"t","timeout_secs":7,"max_rps":3}"#,
        ))
        .expect("runtime");
        assert!(rt.is_login());
        assert_eq!(rt.config.defaults.timeout_secs, 7);
        assert_eq!(rt.config.defaults.max_concurrency, 3);
        assert_eq!(rt.accounts.len(), 1);
        assert_eq!(rt.accounts[0].id, LOGIN_ID);
    }

    #[test]
    fn login_runtime_stays_single_account_under_any_scope() {
        // `AccountScope` torna ilegal combinar `--login` com filtros de escopo;
        // a sessão de login permanece sempre uma única conta sintética.
        let rt = Runtime::from_login(login(r#"{"system":"sinir","token":"t"}"#)).expect("runtime");
        assert!(rt.is_login());
        assert_eq!(rt.accounts.len(), 1);
        assert_eq!(rt.single().expect("single").id, LOGIN_ID);
    }
}
