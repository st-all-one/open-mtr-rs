//! Migração de configuração legada para `config_version = 1`.
use crate::config::CliConfig;
use crate::error::CliError;
use std::path::{Path, PathBuf};

/// Modalidade de execução da migração (mutuamente exclusiva).
#[derive(Clone, Copy, Debug)]
pub enum WriteMode {
    /// Apenas valida e calcula o resultado, sem gravar.
    DryRun,
    /// Grava backup e configuração migrada.
    Apply,
}

/// Resultado de uma migração de configuração.
pub struct MigrateOutcome {
    /// `true` quando a versão precisou ser ajustada.
    pub changed: bool,
    /// Versão encontrada no arquivo, ou `None` se ausente.
    pub from_version: Option<i64>,
    /// Versão resultante (sempre `1`).
    pub to_version: i64,
    /// Caminho do backup criado no modo `Apply` quando houve mudança.
    pub backup: Option<PathBuf>,
    /// `true` no modo `DryRun`, que valida mas não grava nada.
    pub dry_run: bool,
}

/// Migra a configuração em `path` para `config_version = 1`.
pub fn migrate(path: &Path, mode: WriteMode) -> Result<MigrateOutcome, CliError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| CliError::Config(format!("não foi possível ler {}: {e}", path.display())))?;
    let mut doc: toml_edit::DocumentMut = text
        .parse()
        .map_err(|e| CliError::Config(format!("TOML inválido: {e}")))?;

    let current = doc
        .get("config_version")
        .and_then(toml_edit::Item::as_integer);
    if let Some(v) = current
        && v > 1
    {
        return Err(CliError::Config(format!(
            "config_version {v} é mais nova que o binário (suportada: 1); atualize o openmtr"
        )));
    }

    let changed = current != Some(1);
    if changed {
        doc.as_table_mut()
            .insert("config_version", toml_edit::value(1));
    }

    let migrated = doc.to_string();
    let parsed: CliConfig = toml::from_str(&migrated)
        .map_err(|e| CliError::Config(format!("TOML migrado inválido: {e}")))?;
    parsed.validate()?;

    let backup = if changed && matches!(mode, WriteMode::Apply) {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let bak = PathBuf::from(format!("{}.bak.{stamp}", path.display()));
        std::fs::copy(path, &bak)
            .map_err(|e| CliError::Config(format!("criar backup {}: {e}", bak.display())))?;
        std::fs::write(path, &migrated)
            .map_err(|e| CliError::Config(format!("gravar {}: {e}", path.display())))?;
        Some(bak)
    } else {
        None
    };

    Ok(MigrateOutcome {
        changed,
        from_version: current,
        to_version: 1,
        backup,
        dry_run: matches!(mode, WriteMode::DryRun),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, body: &str) -> PathBuf {
        let p = dir.join("cfg.toml");
        std::fs::write(&p, body).expect("write");
        p
    }

    #[test]
    fn dry_run_does_not_touch_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let body = "[[accounts]]\nid = \"a\"\nsystem = \"sinir\"\ntoken = \"t\"\n";
        let p = write(dir.path(), body);
        let out = migrate(&p, WriteMode::DryRun).expect("migrate");
        assert!(out.changed);
        assert_eq!(std::fs::read_to_string(&p).expect("read"), body);
        assert!(out.backup.is_none());
    }

    #[test]
    fn migrate_writes_v1_and_backup() {
        let dir = tempfile::tempdir().expect("tempdir");
        let body = "[[accounts]]\nid = \"a\"\nsystem = \"sinir\"\ntoken = \"t\"\n";
        let p = write(dir.path(), body);
        let out = migrate(&p, WriteMode::Apply).expect("migrate");
        assert!(out.changed);
        assert!(out.backup.is_some());
        let migrated = std::fs::read_to_string(&p).expect("read");
        assert!(migrated.contains("config_version = 1"));
    }

    #[test]
    fn already_current_is_noop() {
        let dir = tempfile::tempdir().expect("tempdir");
        let body =
            "config_version = 1\n[[accounts]]\nid = \"a\"\nsystem = \"sinir\"\ntoken = \"t\"\n";
        let p = write(dir.path(), body);
        let out = migrate(&p, WriteMode::Apply).expect("migrate");
        assert!(!out.changed);
        assert!(out.backup.is_none());
    }

    #[test]
    fn future_version_is_rejected() {
        let dir = tempfile::tempdir().expect("tempdir");
        let body =
            "config_version = 2\n[[accounts]]\nid = \"a\"\nsystem = \"sinir\"\ntoken = \"t\"\n";
        let p = write(dir.path(), body);
        assert!(migrate(&p, WriteMode::DryRun).is_err());
    }
}
