use crate::config::CliConfig;
use crate::error::CliError;
use std::path::{Path, PathBuf};

pub struct MigrateOutcome {
    pub changed: bool,
    pub from_version: Option<i64>,
    pub to_version: i64,
    pub backup: Option<PathBuf>,
    pub dry_run: bool,
}

pub fn migrate(path: &Path, dry_run: bool) -> Result<MigrateOutcome, CliError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| CliError::Config(format!("não foi possível ler {}: {e}", path.display())))?;
    let mut doc: toml_edit::DocumentMut = text
        .parse()
        .map_err(|e| CliError::Config(format!("TOML inválido: {e}")))?;

    let current = doc.get("config_version").and_then(|v| v.as_integer());
    if let Some(v) = current
        && v > 1
    {
        return Err(CliError::Config(format!(
            "config_version {v} é mais nova que o binário (suportada: 1); atualize o openmtr"
        )));
    }

    let changed = current != Some(1);
    if changed {
        doc["config_version"] = toml_edit::value(1);
    }

    let migrated = doc.to_string();
    let parsed: CliConfig = toml::from_str(&migrated)
        .map_err(|e| CliError::Config(format!("TOML migrado inválido: {e}")))?;
    parsed.validate()?;

    let mut backup = None;
    if changed && !dry_run {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let bak = PathBuf::from(format!("{}.bak.{stamp}", path.display()));
        std::fs::copy(path, &bak)
            .map_err(|e| CliError::Config(format!("criar backup {}: {e}", bak.display())))?;
        std::fs::write(path, &migrated)
            .map_err(|e| CliError::Config(format!("gravar {}: {e}", path.display())))?;
        backup = Some(bak);
    }

    Ok(MigrateOutcome {
        changed,
        from_version: current,
        to_version: 1,
        backup,
        dry_run,
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
        let out = migrate(&p, true).expect("migrate");
        assert!(out.changed);
        assert_eq!(std::fs::read_to_string(&p).expect("read"), body);
        assert!(out.backup.is_none());
    }

    #[test]
    fn migrate_writes_v1_and_backup() {
        let dir = tempfile::tempdir().expect("tempdir");
        let body = "[[accounts]]\nid = \"a\"\nsystem = \"sinir\"\ntoken = \"t\"\n";
        let p = write(dir.path(), body);
        let out = migrate(&p, false).expect("migrate");
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
        let out = migrate(&p, false).expect("migrate");
        assert!(!out.changed);
        assert!(out.backup.is_none());
    }

    #[test]
    fn future_version_is_rejected() {
        let dir = tempfile::tempdir().expect("tempdir");
        let body =
            "config_version = 2\n[[accounts]]\nid = \"a\"\nsystem = \"sinir\"\ntoken = \"t\"\n";
        let p = write(dir.path(), body);
        assert!(migrate(&p, true).is_err());
    }
}
