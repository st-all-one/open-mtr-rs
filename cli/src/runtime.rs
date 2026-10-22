use crate::account::{Account, filter_accounts, load_accounts};
use crate::config::CliConfig;
use crate::error::CliError;
use std::path::Path;

pub struct Runtime {
    pub config: CliConfig,
    pub accounts: Vec<Account>,
}

impl Runtime {
    pub fn load(config_path: &Path) -> Result<Self, CliError> {
        let config = CliConfig::load(config_path)?;
        let accounts = load_accounts(&config)?;
        Ok(Self { config, accounts })
    }

    pub fn resolve<'a>(
        &'a self,
        ids: &[String],
        system: Option<&str>,
        tags: &[String],
        all: bool,
    ) -> Result<Vec<&'a Account>, CliError> {
        filter_accounts(&self.accounts, ids, system, tags, all)
    }
}
