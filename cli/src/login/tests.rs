use super::*;

const SINIR: &str = r#"{"system":"sinir","token":"super-secret"}"#;
const MTRPE: &str = r#"{
    "system":"mtr_pe",
    "cnpj":"11222333000181",
    "cpf":"07800211428",
    "unidade":3615,
    "senha":"pw"
}"#;

#[test]
fn sinir_accepts_token() {
    let login = Login::parse(SINIR).expect("login");
    let account = login.into_account().expect("conta");
    assert_eq!(account.system, System::Sinir);
    assert_eq!(account.token.as_deref(), Some("super-secret"));
    assert_eq!(account.cpf, None);
}

#[test]
fn sinir_rejects_classic_credentials() {
    let raw = r#"{"system":"sinir","token":"t","cpf":"1","unidade":1,"senha":"p"}"#;
    assert!(Login::parse(raw).is_err());
}

#[test]
fn sinir_requires_token() {
    assert!(Login::parse(r#"{"system":"sinir"}"#).is_err());
}

#[test]
fn mtrpe_accepts_complete_login() {
    let login = Login::parse(MTRPE).expect("login");
    let account = login.into_account().expect("conta");
    assert_eq!(account.system, System::MtrPe);
    assert_eq!(account.cnpj.as_deref(), Some("11222333000181"));
    assert_eq!(account.unidade, Some(3615));
    assert_eq!(account.senha.as_deref(), Some("pw"));
    assert_eq!(account.token, None);
}

#[test]
fn mtrpe_requires_each_field() {
    for raw in [
        r#"{"system":"mtr_pe","cpf":"1","unidade":1,"senha":"p"}"#,
        r#"{"system":"mtr_pe","cnpj":"1","unidade":1,"senha":"p"}"#,
        r#"{"system":"mtr_pe","cnpj":"1","cpf":"1","senha":"p"}"#,
        r#"{"system":"mtr_pe","cnpj":"1","cpf":"1","unidade":1}"#,
    ] {
        assert!(
            Login::parse(raw).is_err(),
            "aceitou login incompleto: {raw}"
        );
    }
}

#[test]
fn mtrpe_rejects_token() {
    assert!(Login::parse(r#"{"system":"mtr_pe","token":"t"}"#).is_err());
}

#[test]
fn sigor_accepts_token_or_classic() {
    let token = Login::parse(r#"{"system":"sigor","token":"t"}"#).expect("token");
    assert_eq!(token.into_account().expect("conta").system, System::Sigor);
    let classic =
        Login::parse(r#"{"system":"sigor","cpf":"1","unidade":2,"senha":"p"}"#).expect("clássico");
    let account = classic.into_account().expect("conta");
    assert_eq!(account.cnpj, None);
}

#[test]
fn sigor_rejects_mixed_auth() {
    let raw = r#"{"system":"sigor","token":"t","cpf":"1","unidade":2,"senha":"p"}"#;
    assert!(Login::parse(raw).is_err());
}

#[test]
fn unknown_or_adapterless_system_is_rejected() {
    assert!(Login::parse(r#"{"system":"desconhecido","token":"t"}"#).is_err());
    assert!(Login::parse(r#"{"system":"mtr_ce","token":"t"}"#).is_err());
}

#[test]
fn token_and_env_are_exclusive() {
    let raw = r#"{"system":"sinir","token":"t","token_env":"T"}"#;
    assert!(Login::parse(raw).is_err());
}

#[test]
fn debug_redacts_secrets() {
    let login = Login::parse(SINIR).expect("login");
    let text = format!("{login:?}");
    assert!(!text.contains("super-secret"), "vazou token: {text}");
    assert!(text.contains("<redacted>"));
}

#[test]
fn reads_login_from_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("login.json");
    std::fs::write(&path, SINIR).expect("write");
    Login::parse(&format!("@{}", path.display())).expect("login");
}

#[test]
fn missing_env_is_an_error() {
    let login = Login::parse(r#"{"system":"sinir","token_env":"OPENMTR_MISSING_VAR_9E3A1C"}"#)
        .expect("login");
    assert!(login.into_account().is_err());
}

#[test]
fn rejects_blank_token_and_senha() {
    assert!(Login::parse(r#"{"system":"sinir","token":"   "}"#).is_err());
    assert!(Login::parse(r#"{"system":"sinir","token_env":"  "}"#).is_err());
    assert!(
        Login::parse(r#"{"system":"mtr_pe","cnpj":"1","cpf":"1","unidade":1,"senha":""}"#).is_err()
    );
    assert!(Login::parse(r#"{"system":"sigor","cpf":"1","unidade":1,"senha_env":" "}"#).is_err());
}

#[test]
fn invalid_login_error_does_not_echo_secret() {
    let err =
        Login::parse(r#"{"system":"sinir","token":"super-secret",}"#).expect_err("json inválido");
    assert!(!err.to_string().contains("super-secret"));
}

#[test]
fn into_account_applies_limits() {
    let raw = r#"{
        "system":"sinir","token":"t","timeout_secs":7,"max_rps":3,"base_url":"http://x"
    }"#;
    let account = Login::parse(raw)
        .expect("login")
        .into_account()
        .expect("conta");
    assert_eq!(account.max_concurrency, Some(3));
    assert_eq!(account.base_url.as_deref(), Some("http://x"));
}
