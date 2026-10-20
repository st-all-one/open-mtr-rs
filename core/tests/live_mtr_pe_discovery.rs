#![allow(
    clippy::as_conversions,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::pedantic,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::unwrap_used,
    clippy::wildcard_imports,
    reason = "suíte/fixture legada: atalhos de teste não fazem parte da API de produção"
)]

use std::sync::OnceLock;

static ENV_LOADED: OnceLock<()> = OnceLock::new();

fn load_env() {
    ENV_LOADED.get_or_init(|| {
        dotenvy::from_filename_override(".env").ok();
        dotenvy::from_filename_override("../.env").ok();
    });
}

fn env_str(key: &str) -> String {
    load_env();
    std::env::var(key).unwrap_or_else(|_| panic!("{key} not set in .env"))
}

const BASE: &str = "https://mtr.cprh.pe.gov.br/api";

fn reqwest_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("reqwest Client")
}

async fn get_token() -> String {
    load_env();
    let cnpj = env_str("MTR_PE_CNPJ");
    let cpf = env_str("MTR_PE_CPF_USER");
    let password = env_str("MTR_PE_PASSWORD");
    let unity: u32 = env_str("MTR_PE_UNITY")
        .parse()
        .expect("MTR_PE_UNITY must be a number");
    let client = reqwest_client();

    let body = serde_json::json!({
        "pessoaCodigo": unity,
        "pessoaCnpj": cnpj,
        "usuarioCpf": cpf,
        "senha": password,
    });

    let url = format!("{BASE}/gettoken");
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .expect("gettoken failed");
    let status = resp.status();
    let json: serde_json::Value = resp.json().await.expect("gettoken response not JSON");
    assert!(status.is_success(), "gettoken failed");

    json["token"].as_str().expect("no token field").to_string()
}

fn print_endpoint(name: &str, json: &serde_json::Value) {
    println!("\n─── {name} ───");
    if let Some(arr) = json.as_array() {
        println!("  Tipo: ARRAY, items: {}", arr.len());
        if !arr.is_empty() {
            println!(
                "  Keys: {:?}",
                arr[0]
                    .as_object()
                    .map(|o| o.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default()
            );
            println!(
                "  Primeiro: {}",
                serde_json::to_string_pretty(&arr[0]).unwrap_or_default()
            );
        }
    } else {
        println!(
            "  {}",
            serde_json::to_string_pretty(json).unwrap_or_default()
        );
    }
}

fn has_env() -> bool {
    load_env();
    std::env::var("MTR_PE_CNPJ").is_ok()
}

// ─── Testes de Descoberta (ignorados por padrão, requerem .env) ──────────

#[tokio::test]
async fn discover_01_gettoken() {
    if !has_env() {
        eprintln!("SKIP: MTR_PE_CNPJ not set");
        return;
    }
    let token = get_token().await;
    println!("Token: {}...", &token[..40]);
    assert!(token.len() > 20);
}

#[tokio::test]
async fn discover_02_classes() {
    if !has_env() {
        eprintln!("SKIP: MTR_PE_CNPJ not set");
        return;
    }
    let token = get_token().await;
    let client = reqwest_client();
    let resp: serde_json::Value = client
        .post(format!("{BASE}/retornaListaClasse"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    print_endpoint("retornaListaClasse", &resp);
    assert!(resp.is_array());
}

#[tokio::test]
async fn discover_03_unidades() {
    if !has_env() {
        eprintln!("SKIP");
        return;
    }
    let token = get_token().await;
    let client = reqwest_client();
    let resp: serde_json::Value = client
        .post(format!("{BASE}/retornaListaUnidade"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    print_endpoint("retornaListaUnidade", &resp);
    assert!(resp.is_array());
}

#[tokio::test]
async fn discover_04_tecnologias() {
    if !has_env() {
        eprintln!("SKIP");
        return;
    }
    let token = get_token().await;
    let client = reqwest_client();
    let resp: serde_json::Value = client
        .post(format!("{BASE}/retornaListaTecnologia"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    print_endpoint("retornaListaTecnologia", &resp);
    assert!(resp.is_array());
}

#[tokio::test]
async fn discover_05_estados_fisicos() {
    if !has_env() {
        eprintln!("SKIP");
        return;
    }
    let token = get_token().await;
    let client = reqwest_client();
    let resp: serde_json::Value = client
        .post(format!("{BASE}/retornaListaEstadoFisico"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    print_endpoint("retornaListaEstadoFisico", &resp);
    assert!(resp.is_array());
}

#[tokio::test]
async fn discover_06_residuos() {
    if !has_env() {
        eprintln!("SKIP");
        return;
    }
    let token = get_token().await;
    let client = reqwest_client();
    let resp: serde_json::Value = client
        .post(format!("{BASE}/retornaListaResiduo"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    print_endpoint("retornaListaResiduo", &resp);
    assert!(resp.is_array());
}

#[tokio::test]
async fn discover_07_acondicionamentos() {
    if !has_env() {
        eprintln!("SKIP");
        return;
    }
    let token = get_token().await;
    let client = reqwest_client();
    let resp: serde_json::Value = client
        .post(format!("{BASE}/retornaListaAcondicionamento"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    print_endpoint("retornaListaAcondicionamento", &resp);
    assert!(resp.is_array());
}

#[tokio::test]
async fn discover_11_lista_barras_periodo() {
    if !has_env() {
        eprintln!("SKIP");
        return;
    }
    let token = get_token().await;
    let client = reqwest_client();
    let url = format!("{BASE}/retornaListaCodigoBarasManifesto/20260629/20260729");
    let resp: serde_json::Value = client
        .post(&url)
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    print_endpoint("retornaListaCodigoBarasManifesto", &resp);
}
