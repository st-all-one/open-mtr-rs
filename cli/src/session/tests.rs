use super::*;
use open_mtr_core::openmtr::system::System;
use tempfile::tempdir;

fn account(token: &str) -> Account {
    Account {
        id: "login".to_string(),
        label: None,
        system: System::Sinir,
        tags: Vec::new(),
        access_type: "geradora".to_string(),
        access_priority: 2,
        base_url: Some("http://example.test".to_string()),
        token: Some(token.to_string()),
        token_env: None,
        cpf: None,
        senha: None,
        senha_env: None,
        unidade: None,
        cnpj: None,
        max_concurrency: None,
        enabled: true,
    }
}

#[test]
fn persists_across_instances_and_respects_expiry() {
    let dir = tempdir().expect("tempdir");
    let account = account("refresh-token");
    let cache = SessionCache::with_dir(dir.path());
    cache.put(&account, "access-token");

    let other = SessionCache::with_dir(dir.path());
    assert_eq!(other.get(&account).as_deref(), Some("access-token"));

    let expired = CacheEntry {
        token: "old".to_string(),
        expires_at: now_unix().saturating_sub(1),
    };
    let third = SessionCache::with_dir(dir.path());
    third.write_file(&key_for(&account), &expired);
    assert!(third.get(&account).is_none());
}

#[test]
fn key_changes_with_credentials() {
    assert_ne!(key_for(&account("a")), key_for(&account("b")));
}

#[cfg(unix)]
#[test]
fn cache_files_are_private() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().expect("tempdir");
    let account = account("refresh-token");
    let cache = SessionCache::with_dir(dir.path());
    cache.put(&account, "access-token");
    let path = dir.path().join(format!("{}.json", key_for(&account)));
    let mode = std::fs::metadata(path)
        .expect("metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}
