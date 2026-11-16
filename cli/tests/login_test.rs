use clap::Parser;
use open_mtr_cli::account::Account;
use open_mtr_cli::batch::{BatchOptions, run_to};
use open_mtr_cli::cli::Cli;
use open_mtr_cli::config::{CliConfig, Defaults};
use open_mtr_cli::login::Login;
use open_mtr_cli::runtime::Runtime;
use open_mtr_cli::session::SessionCache;
use open_mtr_core::openmtr::system::System;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn runtime(raw: &str) -> Runtime {
    let mut runtime = Runtime::from_login(Login::parse(raw).expect("login")).expect("runtime");
    runtime.cache = SessionCache::memory();
    runtime
}

#[tokio::test]
async fn login_runs_batch_without_config_or_account() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "erro": false,
            "objetoResposta": "Bearer jwt",
            "totalRecords": 0,
            "mensagem": null
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/listaManifesto"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "erro": false,
            "objetoResposta": [{ "manNumero": "1" }],
            "totalRecords": 1
        })))
        .mount(&server)
        .await;

    let raw = format!(
        r#"{{"system":"sinir","token":"t","base_url":"{}"}}"#,
        server.uri()
    );
    let rt = runtime(&raw);
    assert!(rt.is_login());
    assert_eq!(rt.accounts[0].id, open_mtr_cli::login::LOGIN_ID);

    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("ops.jsonl");
    std::fs::write(
        &file,
        "{\"op\":\"mtr.consultar\",\"params\":{\"periodo\":{\"dataInicial\":\"01-05-2026\",\"dataFinal\":\"31-05-2026\"}}}\n",
    )
    .expect("write");
    let opts = BatchOptions {
        file: Some(file),
        continue_on_error: false,
        dry_run: false,
        fixed_time: None,
    };
    let mut out = Vec::new();
    let result = run_to(&rt, &opts, &mut out).await;
    assert!(
        result.is_ok(),
        "batch: {result:?}; saida={}",
        String::from_utf8_lossy(&out)
    );
    let text = String::from_utf8(out).expect("utf8");
    let line: serde_json::Value =
        serde_json::from_str(text.lines().next().expect("linha")).expect("json");
    assert_eq!(line["ok"], true);
    assert_eq!(line["data"]["dados"].as_array().expect("dados").len(), 1);
}

#[tokio::test]
async fn batch_reuses_access_token_within_ttl() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "erro": false,
            "objetoResposta": "Bearer cached-jwt",
            "totalRecords": 0,
            "mensagem": null
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/listaManifesto"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "erro": false,
            "objetoResposta": [{ "manNumero": "1" }],
            "totalRecords": 1
        })))
        .expect(2)
        .mount(&server)
        .await;

    let raw = format!(
        r#"{{"system":"sinir","token":"refresh","base_url":"{}"}}"#,
        server.uri()
    );
    let mut rt = runtime(&raw);
    let dir = tempfile::tempdir().expect("tempdir");
    rt.cache = SessionCache::with_dir(dir.path());

    let ops = format!(
        "{}\n{}\n",
        r#"{"op":"mtr.consultar","params":{"periodo":{"dataInicial":"01-05-2026","dataFinal":"31-05-2026"}}}"#,
        r#"{"op":"mtr.consultar","params":{"periodo":{"dataInicial":"01-05-2026","dataFinal":"31-05-2026"}}}"#
    );
    let file = dir.path().join("ops.jsonl");
    std::fs::write(&file, ops).expect("write");
    let opts = BatchOptions {
        file: Some(file),
        continue_on_error: false,
        dry_run: false,
        fixed_time: None,
    };
    let mut out = Vec::new();
    run_to(&rt, &opts, &mut out).await.expect("batch");

    let lines: Vec<serde_json::Value> = String::from_utf8(out)
        .expect("utf8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("json"))
        .collect();
    assert_eq!(lines.len(), 2);
    server.verify().await;
}

#[tokio::test]
async fn classic_mtrpe_login_is_cached_between_clients() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/gettoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "token": "jwt" })))
        .expect(1)
        .mount(&server)
        .await;

    let account = Account {
        id: "mtrpe".to_string(),
        label: None,
        system: System::MtrPe,
        tags: Vec::new(),
        access_type: "geradora".to_string(),
        access_priority: 2,
        base_url: Some(server.uri()),
        token: None,
        token_env: None,
        cpf: Some("07800211428".to_string()),
        senha: Some("pw".to_string()),
        senha_env: None,
        unidade: Some(3615),
        cnpj: Some("11222333000181".to_string()),
        max_concurrency: Some(1),
        enabled: true,
    };
    let cfg = CliConfig {
        config_version: 1,
        defaults: Defaults::default(),
        accounts: Vec::new(),
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let cache = SessionCache::with_dir(dir.path());

    account
        .build_client(&cfg, None, &cache)
        .await
        .expect("primeiro cliente");
    account
        .build_client(&cfg, None, &cache)
        .await
        .expect("cliente cacheado");

    server.verify().await;
}

#[test]
fn login_rejects_scope_flags() {
    let rt = runtime(r#"{"system":"sinir","token":"t"}"#);
    assert!(rt.select(&["outra".to_string()], None, &[], false).is_err());
    assert!(rt.select(&[], Some("sinir"), &[], false).is_err());
    assert!(rt.select(&[], None, &["tag".to_string()], false).is_err());
    assert!(rt.select(&[], None, &[], true).is_err());
    assert_eq!(rt.select(&[], None, &[], false).expect("login").len(), 1);
}

#[tokio::test]
async fn batch_requires_explicit_file() {
    let rt = runtime(r#"{"system":"sinir","token":"t"}"#);
    let opts = BatchOptions {
        file: None,
        continue_on_error: false,
        dry_run: true,
        fixed_time: None,
    };
    let mut out = Vec::new();
    let err = run_to(&rt, &opts, &mut out).await;
    assert!(err.is_err(), "batch sem --file deve falhar");
}

#[tokio::test]
async fn login_is_rejected_for_config_account_and_self() {
    let login = r#"{"system":"sinir","token":"t"}"#;
    for args in [
        vec!["openmtr", "--login", login, "config", "validate"],
        vec!["openmtr", "--login", login, "account", "list"],
        vec!["openmtr", "--login", login, "self", "version"],
    ] {
        let cli = Cli::try_parse_from(args).expect("parse");
        let err = cli.run().await.expect_err("deve rejeitar --login");
        assert!(err.to_string().contains("--login"), "erro: {err}");
    }
}

#[tokio::test]
async fn login_stdin_conflicts_with_params_or_file_stdin() {
    for args in [
        vec![
            "openmtr",
            "--login",
            "-",
            "mtr",
            "consultar",
            "--params",
            "-",
        ],
        vec!["openmtr", "--login", "-", "batch", "--file", "-"],
    ] {
        let cli = Cli::try_parse_from(args).expect("parse");
        let err = cli.run().await.expect_err("conflito de stdin");
        assert!(err.to_string().contains("stdin"), "erro: {err}");
    }
}

#[tokio::test]
async fn login_write_rejects_account() {
    let cli = Cli::try_parse_from([
        "openmtr",
        "--login",
        r#"{"system":"sinir","token":"t"}"#,
        "mtr",
        "criar",
        "--account",
        "prod",
        "--params",
        "{}",
    ])
    .expect("parse");
    let err = cli.run().await.expect_err("conflito");
    assert!(err.to_string().contains("--account"), "erro: {err}");
}

#[tokio::test]
async fn batch_login_rejects_account_field() {
    let rt = runtime(r#"{"system":"sinir","token":"t"}"#);
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("ops.jsonl");
    std::fs::write(
        &file,
        "{\"op\":\"mtr.consultar\",\"account\":\"prod\",\"params\":{}}\n",
    )
    .expect("write");
    let opts = BatchOptions {
        file: Some(file),
        continue_on_error: false,
        dry_run: true,
        fixed_time: None,
    };
    let mut out = Vec::new();
    run_to(&rt, &opts, &mut out)
        .await
        .expect_err("conflito de account");
    let text = String::from_utf8(out).expect("utf8");
    assert!(text.contains("--login"), "saida: {text}");
}

#[tokio::test]
async fn batch_dry_run_covers_new_ops() {
    let rt = runtime(r#"{"system":"sinir","token":"t"}"#);
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("ops.jsonl");
    std::fs::write(
        &file,
        concat!(
            "{\"op\":\"mtr.criar\",\"params\":[{}]}\n",
            "{\"op\":\"cdf.gerar\",\"params\":[{}]}\n",
            "{\"op\":\"ref.listar\",\"params\":{\"table\":\"estado-fisico\"}}\n",
            "{\"op\":\"ref.por-residuo\",\"params\":{\"codigo\":\"F001\"}}\n",
            "{\"op\":\"ref.por-estado-fisico\",\"params\":{\"codigo\":\"1\"}}\n",
            "{\"op\":\"mtr.download\",\"params\":{\"numeros\":[\"1\"]},\"out\":\"x\"}\n",
        ),
    )
    .expect("write");
    let opts = BatchOptions {
        file: Some(file),
        continue_on_error: false,
        dry_run: true,
        fixed_time: None,
    };
    let mut out = Vec::new();
    run_to(&rt, &opts, &mut out).await.expect("batch");
    let lines: Vec<serde_json::Value> = String::from_utf8(out)
        .expect("utf8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("json"))
        .collect();
    assert_eq!(lines.len(), 6);
    for line in lines {
        assert_eq!(line["ok"], true);
        assert_eq!(line["data"]["dry_run"], true);
    }
}

#[test]
fn login_account_builds_from_mtrpe_credentials() {
    let rt = runtime(
        r#"{
            "system":"mtr_pe",
            "cnpj":"11222333000181",
            "cpf":"07800211428",
            "unidade":3615,
            "senha":"pw"
        }"#,
    );
    let account = &rt.accounts[0];
    assert_eq!(account.system_slug(), "mtr_pe");
    assert_eq!(account.cnpj.as_deref(), Some("11222333000181"));
    assert_eq!(account.unidade, Some(3615));
    assert_eq!(account.senha.as_deref(), Some("pw"));
}
