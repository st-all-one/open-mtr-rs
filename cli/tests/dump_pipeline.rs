use open_mtr_cli::account::Account;
use open_mtr_cli::config::{CliConfig, Defaults, RawAccount};
use open_mtr_cli::dump::{DumpOptions, run};
use open_mtr_cli::normalize::CSV_HEADER;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn raw(id: &str, access_type: &str, base_url: &str) -> RawAccount {
    RawAccount {
        id: id.to_string(),
        label: None,
        system: "sinir".to_string(),
        tags: vec!["test".to_string()],
        access_type: Some(access_type.to_string()),
        base_url: Some(base_url.to_string()),
        token: Some("test-token".to_string()),
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

fn detail(numero: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "erro": false,
        "objetoResposta": {
            "manNumero": numero,
            "situacaoManifesto": { "simDescricao": "Ativo" },
            "parceiroGerador": { "parCnpj": "11222333000181", "parDescricao": "Geradora" },
            "parceiroTransportador": { "parCnpj": "11222333000181", "parDescricao": "Transportadora" },
            "parceiroDestinador": { "parCnpj": "11222333000181", "parDescricao": "Destinadora" },
            "listaManifestoResiduo": [{
                "residuo": { "resCodigoIbama": "F001", "resDescricao": "Resíduo" },
                "marQuantidade": 1.5
            }]
        }
    }))
}

#[tokio::test]
async fn dump_deduplicates_same_mtr_across_accounts() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "erro": false,
            "objetoResposta": "Bearer test-jwt-token",
            "totalRecords": 0,
            "mensagem": null
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/listaManifesto"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "erro": false,
            "objetoResposta": [{ "manNumero": "111" }, { "manNumero": "222" }],
            "totalRecords": 2
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/retornaManifesto/111"))
        .respond_with(detail("111"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/retornaManifesto/222"))
        .respond_with(detail("222"))
        .mount(&server)
        .await;

    let cfg = CliConfig {
        config_version: 1,
        defaults: Defaults::default(),
        accounts: vec![
            raw("sinir-geradora", "geradora", &server.uri()),
            raw("sinir-transportadora", "transportadora", &server.uri()),
        ],
    };
    let accounts: Vec<Account> = cfg
        .accounts
        .iter()
        .map(|r| Account::from_raw(r).expect("conta"))
        .collect();
    let refs: Vec<&Account> = accounts.iter().collect();

    let dir = tempfile::tempdir().expect("tempdir");
    let opts = DumpOptions {
        from: "2026-05-01".parse_date_helper(),
        to: "2026-05-31".parse_date_helper(),
        window_days: 30,
        out: dir.path().to_path_buf(),
        pdf: false,
        no_dedup: false,
        resume: false,
        force: false,
        format: "both".to_string(),
        concurrency: 2,
        fixed_time: Some(
            chrono::DateTime::parse_from_rfc3339("2026-06-01T00:00:00Z")
                .expect("data")
                .with_timezone(&chrono::Utc),
        ),
    };
    run(&cfg, &refs, &opts).await.expect("dump");

    let lines = std::fs::read_to_string(dir.path().join("mtrs.jsonl")).expect("jsonl");
    let rows: Vec<serde_json::Value> = lines
        .lines()
        .map(|l| serde_json::from_str(l).expect("linha"))
        .collect();
    assert_eq!(rows.len(), 2, "dedup deve emitir um registro por MTR");
    assert!(rows.iter().all(|r| r["account"] == "sinir-transportadora"));

    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("relatorio.json")).expect("rel"),
    )
    .expect("json");
    assert_eq!(report["totals"]["emitted"], 2);
    assert_eq!(report["totals"]["deduplicated"], 2);

    let csv = std::fs::read_to_string(dir.path().join("mtrs.csv")).expect("csv");
    let mut lines = csv.lines();
    assert_eq!(lines.next(), Some(CSV_HEADER));
    assert_eq!(lines.count(), 2, "duas linhas de MTR");
    assert!(csv.contains("Geradora"));
    assert!(csv.contains("Destinadora"));
    assert!(csv.contains("F001"));
}

#[allow(
    clippy::excessive_nesting,
    reason = "teste monta dois dumps e compara arquivos"
)]
#[tokio::test]
async fn dump_is_byte_deterministic_with_fixed_time() {
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
            "objetoResposta": [{ "manNumero": "111" }, { "manNumero": "222" }],
            "totalRecords": 2
        })))
        .mount(&server)
        .await;
    for numero in ["111", "222"] {
        Mock::given(method("GET"))
            .and(path(format!("/retornaManifesto/{numero}")))
            .respond_with(detail(numero))
            .mount(&server)
            .await;
    }

    let cfg = CliConfig {
        config_version: 1,
        defaults: Defaults::default(),
        accounts: vec![raw("sinir", "transportadora", &server.uri())],
    };
    let accounts: Vec<Account> = cfg
        .accounts
        .iter()
        .map(|r| Account::from_raw(r).expect("conta"))
        .collect();
    let refs: Vec<&Account> = accounts.iter().collect();
    let fixed = chrono::DateTime::parse_from_rfc3339("2026-06-01T00:00:00Z")
        .expect("data")
        .with_timezone(&chrono::Utc);

    let mut outputs = Vec::new();
    for _ in 0..2 {
        let dir = tempfile::tempdir().expect("tempdir");
        let opts = DumpOptions {
            from: "2026-05-01".parse_date_helper(),
            to: "2026-05-31".parse_date_helper(),
            window_days: 30,
            out: dir.path().to_path_buf(),
            pdf: false,
            no_dedup: false,
            resume: false,
            force: false,
            format: "both".to_string(),
            concurrency: 2,
            fixed_time: Some(fixed),
        };
        run(&cfg, &refs, &opts).await.expect("dump");
        let mut files = Vec::new();
        for name in ["mtrs.jsonl", "mtrs.csv", "residuos.csv", "relatorio.json"] {
            files.push(std::fs::read(dir.path().join(name)).expect("arquivo"));
        }
        outputs.push(files);
    }
    for (a, b) in outputs[0].iter().zip(outputs[1].iter()) {
        assert_eq!(a, b, "saída determinística deve ser idêntica");
    }
}

trait DateHelper {
    fn parse_date_helper(&self) -> chrono::NaiveDate;
}

impl DateHelper for &str {
    fn parse_date_helper(&self) -> chrono::NaiveDate {
        chrono::NaiveDate::parse_from_str(self, "%Y-%m-%d").expect("data")
    }
}
