//! Cache curto de access tokens por conta.
//!
//! Evita repetir login/refresh em cada operação de um batch e entre invocações
//! próximas da CLI. O TTL padrão é de 5 minutos; tokens JWT com `exp` menor
//! respeitam o vencimento (com 30 s de folga). O arquivo contém apenas o access
//! token e o instante de expiração; credenciais nunca são gravadas.

use crate::account::Account;
use open_mtr_core::openmtr::client::token::TokenManager;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const SESSION_TTL_SECS: u64 = 5 * 60;
const EXPIRY_SKEW_SECS: u64 = 30;
const MIN_CACHE_SECS: u64 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    token: String,
    expires_at: u64,
}

/// Cache em memória + arquivo, chaveado por conta e credenciais.
///
/// O `Mutex` protege apenas o mapa em memória; o lock nunca é mantido durante
/// I/O ou `.await`.
pub struct SessionCache {
    dir: Option<PathBuf>,
    memory: Mutex<HashMap<String, CacheEntry>>,
}

impl SessionCache {
    /// Cache com o diretório padrão do ambiente (`XDG_RUNTIME_DIR`,
    /// `XDG_CACHE_HOME` ou `~/.cache`). `OPENMTR_NO_SESSION_CACHE=1` desliga
    /// a persistência em disco (o cache em memória continua ativo).
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            dir: default_dir(),
            memory: Mutex::new(HashMap::new()),
        }
    }

    /// Cache apenas em memória (testes e execuções sem diretório gravável).
    #[must_use]
    pub fn memory() -> Self {
        Self::with_dir_option(None)
    }

    /// Cache com diretório explícito.
    #[must_use]
    pub fn with_dir(dir: impl Into<PathBuf>) -> Self {
        Self::with_dir_option(Some(dir.into()))
    }

    fn with_dir_option(dir: Option<PathBuf>) -> Self {
        Self {
            dir,
            memory: Mutex::new(HashMap::new()),
        }
    }

    /// Devolve o access token em cache, se ainda válido.
    #[must_use]
    pub fn get(&self, account: &Account) -> Option<String> {
        let key = key_for(account);
        let now = now_unix();
        if let Some(token) = self.memory_get(&key, now) {
            return Some(token);
        }
        let entry = self.read_file(&key)?;
        if entry.expires_at <= now {
            self.remove_file(&key);
            return None;
        }
        if let Ok(mut memory) = self.memory.lock() {
            memory.insert(key, entry.clone());
        }
        Some(entry.token)
    }

    fn memory_get(&self, key: &str, now: u64) -> Option<String> {
        let mut memory = self.memory.lock().ok()?;
        let entry = memory.get(key).cloned()?;
        if entry.expires_at > now {
            return Some(entry.token);
        }
        memory.remove(key);
        None
    }

    /// Guarda um access token; tokens vazios ou já vencidos são ignorados.
    pub fn put(&self, account: &Account, token: &str) {
        if token.trim().is_empty() {
            return;
        }
        let now = now_unix();
        let ttl = match TokenManager::extract_expiry(token) {
            Some(exp) => exp
                .saturating_sub(EXPIRY_SKEW_SECS)
                .saturating_sub(now)
                .min(SESSION_TTL_SECS),
            None => SESSION_TTL_SECS,
        };
        if ttl <= MIN_CACHE_SECS {
            return;
        }
        let entry = CacheEntry {
            token: token.to_string(),
            expires_at: now + ttl,
        };
        let key = key_for(account);
        if let Ok(mut memory) = self.memory.lock() {
            memory.insert(key.clone(), entry.clone());
        }
        self.write_file(&key, &entry);
    }

    fn read_file(&self, key: &str) -> Option<CacheEntry> {
        let dir = self.dir.as_ref()?;
        let raw = std::fs::read(dir.join(format!("{key}.json"))).ok()?;
        serde_json::from_slice(&raw).ok()
    }

    fn write_file(&self, key: &str, entry: &CacheEntry) {
        let Some(dir) = self.dir.as_ref() else {
            return;
        };
        if let Err(error) = Self::write_file_inner(dir, key, entry) {
            tracing::debug!(%error, "não foi possível gravar o cache de sessão");
        }
    }

    fn write_file_inner(dir: &Path, key: &str, entry: &CacheEntry) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
        }
        let data =
            serde_json::to_vec(entry).map_err(|error| std::io::Error::other(error.to_string()))?;
        let path = dir.join(format!("{key}.json"));
        let temp_path = dir.join(format!(".{key}.{}.tmp", std::process::id()));
        let result = (|| -> std::io::Result<()> {
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&temp_path)?;
            file.write_all(&data)?;
            file.sync_all()?;
            drop(file);
            std::fs::rename(&temp_path, &path)?;
            Ok(())
        })();
        if result.is_err()
            && let Err(error) = std::fs::remove_file(&temp_path)
        {
            tracing::debug!(%error, "não foi possível limpar o arquivo temporário");
        }
        result
    }

    fn remove_file(&self, key: &str) {
        if let Some(dir) = &self.dir
            && let Err(error) = std::fs::remove_file(dir.join(format!("{key}.json")))
        {
            tracing::debug!(%error, "não foi possível remover o cache expirado");
        }
    }
}

fn default_dir() -> Option<PathBuf> {
    if std::env::var_os("OPENMTR_NO_SESSION_CACHE").is_some_and(|value| {
        let value = value.to_string_lossy();
        !value.is_empty() && value != "0"
    }) {
        return None;
    }
    if let Some(dir) = std::env::var_os("OPENMTR_SESSION_DIR") {
        return Some(PathBuf::from(dir));
    }
    if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        return Some(PathBuf::from(dir).join("openmtr/session"));
    }
    if let Some(dir) = std::env::var_os("XDG_CACHE_HOME") {
        return Some(PathBuf::from(dir).join("openmtr/session"));
    }
    #[cfg(windows)]
    if let Some(dir) = std::env::var_os("LOCALAPPDATA") {
        return Some(PathBuf::from(dir).join("openmtr/session"));
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache/openmtr/session"))
}

fn key_for(account: &Account) -> String {
    let mut hash = Sha256::new();
    field(&mut hash, "openmtr-session-v1");
    field(&mut hash, &account.id);
    field(&mut hash, account.system_slug());
    field(&mut hash, account.base_url.as_deref().unwrap_or_default());
    field(&mut hash, account.token.as_deref().unwrap_or_default());
    field(&mut hash, account.token_env.as_deref().unwrap_or_default());
    field(&mut hash, account.cpf.as_deref().unwrap_or_default());
    field(
        &mut hash,
        &account.unidade.map_or_else(String::new, |u| u.to_string()),
    );
    field(&mut hash, account.cnpj.as_deref().unwrap_or_default());
    field(&mut hash, account.senha.as_deref().unwrap_or_default());
    field(&mut hash, account.senha_env.as_deref().unwrap_or_default());
    hex::encode(hash.finalize())
}

fn field(hash: &mut Sha256, value: &str) {
    let len = u64::try_from(value.len()).unwrap_or(u64::MAX);
    hash.update(len.to_le_bytes());
    hash.update(value.as_bytes());
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[cfg(test)]
mod tests;
