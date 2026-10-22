use super::{DumpOptions, internal, window_key};
use crate::account::Account;
use crate::error::CliError;
use crate::normalize::DumpRow;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Serialize, serde::Deserialize)]
pub(super) struct State {
    version: u32,
    period: String,
    accounts: BTreeMap<String, AccountState>,
}

#[derive(Debug, Default, Serialize, serde::Deserialize)]
struct AccountState {
    windows: BTreeMap<String, bool>,
}

impl State {
    pub(super) fn new(opts: &DumpOptions, accounts: &[&Account]) -> Self {
        Self {
            version: 1,
            period: window_key(opts.from, opts.to),
            accounts: accounts
                .iter()
                .map(|a| (a.id.clone(), AccountState::default()))
                .collect(),
        }
    }

    pub(super) fn is_finished(&self, account: &str, key: &str) -> bool {
        self.accounts
            .get(account)
            .and_then(|s| s.windows.get(key))
            .copied()
            .unwrap_or(false)
    }

    pub(super) fn mark(&mut self, account: &str, key: &str) {
        self.accounts
            .entry(account.to_string())
            .or_default()
            .windows
            .insert(key.to_string(), true);
    }
}

pub(super) fn load_state(
    path: &Path,
    opts: &DumpOptions,
    accounts: &[&Account],
) -> Result<State, CliError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| CliError::Config(format!("ler estado de retomada: {e}")))?;
    let state: State = serde_json::from_str(&text)
        .map_err(|e| CliError::Config(format!("estado de retomada inválido: {e}")))?;
    let expected = State::new(opts, accounts);
    if state.version != 1 || state.period != expected.period {
        return Err(CliError::Conflict(
            "estado de retomada pertence a outro período; use --force ou outro --out".into(),
        ));
    }
    Ok(state)
}

pub(super) fn save_state(path: &Path, state: &State) -> Result<(), CliError> {
    let temp = path.with_extension("json.tmp");
    std::fs::write(
        &temp,
        serde_json::to_string_pretty(state).map_err(internal)?,
    )
    .map_err(|e| CliError::Config(format!("gravar estado: {e}")))?;
    std::fs::rename(&temp, path).map_err(|e| CliError::Config(format!("renomear estado: {e}")))
}

pub(super) fn load_previous_rows(out: &Path) -> Result<Vec<DumpRow>, CliError> {
    let path = out.join("mtrs.jsonl");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| CliError::Config(format!("ler JSONL anterior: {e}")))?;
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str(l)
                .map_err(|e| CliError::Config(format!("JSONL anterior inválido: {e}")))
        })
        .collect()
}
