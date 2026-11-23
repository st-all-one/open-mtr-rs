//! Seleção de contas por escopo.

use super::Account;
use crate::error::CliError;
use open_mtr_core::openmtr::system::System;

/// Escopo de seleção de contas (mutuamente exclusivo na CLI).
#[derive(Clone, Copy, Debug, Default)]
pub enum AccountScope<'a> {
    /// Seleção explícita por IDs.
    Ids(&'a [String]),
    /// Seleção por sistema.
    System(&'a str),
    /// Seleção por tags (todas devem corresponder).
    Tags(&'a [String]),
    /// Todas as contas habilitadas.
    #[default]
    All,
}

/// Filtra contas habilitadas pelo escopo selecionado na CLI.
pub fn filter_accounts<'a>(
    accounts: &'a [Account],
    scope: AccountScope<'_>,
) -> Result<Vec<&'a Account>, CliError> {
    let enabled: Vec<&Account> = accounts.iter().filter(|a| a.enabled).collect();
    match scope {
        AccountScope::All => Ok(enabled),
        AccountScope::Ids(ids) => {
            if ids.is_empty() {
                return Ok(Vec::new());
            }
            ids.iter()
                .map(|id| {
                    enabled
                        .iter()
                        .find(|a| a.id == *id)
                        .copied()
                        .ok_or_else(|| CliError::NotFound(format!("conta não encontrada: {id}")))
                })
                .collect()
        }
        AccountScope::System(sys) => {
            let system: System = sys.parse().map_err(CliError::Config)?;
            Ok(enabled.into_iter().filter(|a| a.system == system).collect())
        }
        AccountScope::Tags(tags) => Ok(enabled
            .into_iter()
            .filter(|a| tags.iter().all(|t| a.tags.contains(t)))
            .collect()),
    }
}
