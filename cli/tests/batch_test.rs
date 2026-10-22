use open_mtr_cli::account::Account;
use open_mtr_cli::batch::{BatchOptions, run_to};
use open_mtr_cli::config::{CliConfig, Defaults, RawAccount};
use open_mtr_cli::runtime::Runtime;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn raw(base_url: &str) -> RawAccount {
    RawAccount {
        id: "sinir".to_string(),
        label: None,
        system: "sinir".to_string(),
        tags: vec![],
        access_type: None,
        base_url: Some(base_url.to_string()),
        token: Some("t".to_string()),
        token_env: None,
        cpf: None,
        senha: None,
        senha_env: None,
        unidade: None,
        cnpj: None,
        max_concurrency: Some(2),
        enabled: Some(true),
    }
}

#[tokio::test]
async fn batch_executes_consultar_and_emits_jsonl() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "erro": false, "objetoResposta": "Bearer jwt", "totalRecords": 0, "mensagem": null
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/listaManifesto"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "erro": false,
            "objetoResposta": [{ "manNumero": "1" }, { "manNumero": "2" }],
            "totalRecords": 2
        })))
        .mount(&server)
        .await;

    let cfg = CliConfig {
        config_version: 1,
        defaults: Defaults::default(),
        accounts: vec![raw(&server.uri())],
    };
    let accounts: Vec<Account> = cfg
        .accounts
        .iter()
        .map(|r| Account::from_raw(r).expect("conta"))
        .collect();
    let rt = Runtime {
        config: cfg,
        accounts,
    };

    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("ops.jsonl");
    std::fs::write(
        &file,
        "{\"op\":\"mtr.consultar\",\"account\":\"sinir\",\"params\":{\"periodo\":{\"dataInicial\":\"01-05-2026\",\"dataFinal\":\"31-05-2026\"}}}\n\
         {\"op\":\"health\"}\n",
    )
    .expect("write");

    let opts = BatchOptions {
        file: Some(file),
        continue_on_error: false,
        dry_run: false,
        fixed_time: None,
    };
    let mut out = Vec::new();
    run_to(&rt, &opts, &mut out).await.expect("batch");
    let text = String::from_utf8(out).expect("utf8");
    let lines: Vec<serde_json::Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).expect("json"))
        .collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["index"], 1);
    assert_eq!(lines[0]["ok"], true);
    assert_eq!(
        lines[0]["data"]["dados"].as_array().expect("dados").len(),
        2
    );
    assert_eq!(lines[1]["op"], "health");
}

#[tokio::test]
async fn batch_dry_run_does_not_hit_network() {
    let cfg = CliConfig {
        config_version: 1,
        defaults: Defaults::default(),
        accounts: vec![raw("http://127.0.0.1:9")],
    };
    let accounts: Vec<Account> = cfg
        .accounts
        .iter()
        .map(|r| Account::from_raw(r).expect("conta"))
        .collect();
    let rt = Runtime {
        config: cfg,
        accounts,
    };

    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("ops.jsonl");
    std::fs::write(&file, "{\"params\":{}}\n").expect("write");
    // linha sem 'op' válido deve falhar com schema, não ir à rede
    let opts = BatchOptions {
        file: Some(file),
        continue_on_error: true,
        dry_run: true,
        fixed_time: None,
    };
    let mut out = Vec::new();
    let err = run_to(&rt, &opts, &mut out).await;
    assert!(err.is_err(), "op ausente deve falhar");
}
