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

fn has_env() -> bool {
    load_env();
    std::env::var("MTR_PE_CNPJ").is_ok()
}

const BASE: &str = "https://mtr.cprh.pe.gov.br/api";

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .expect("reqwest Client")
}

async fn get_token() -> String {
    load_env();
    let cnpj = std::env::var("MTR_PE_CNPJ").unwrap();
    let cpf = std::env::var("MTR_PE_CPF_USER").unwrap();
    let password = std::env::var("MTR_PE_PASSWORD").unwrap();
    let unity: u32 = std::env::var("MTR_PE_UNITY").unwrap().parse().unwrap();

    let resp = client()
        .post(format!("{BASE}/gettoken"))
        .json(&serde_json::json!({
            "pessoaCodigo": unity,
            "pessoaCnpj": cnpj,
            "usuarioCpf": cpf,
            "senha": password,
        }))
        .send()
        .await
        .expect("gettoken failed");

    let status = resp.status();
    let json: serde_json::Value = resp.json().await.unwrap();
    assert!(status.is_success(), "gettoken failed: {}", json);
    json["token"].as_str().unwrap().to_string()
}

/// Obtém a lista de códigos de barras em um período específico (máx 30 dias)
async fn listar_barcodes_range(
    token: &str,
    inicio: chrono::NaiveDate,
    fim: chrono::NaiveDate,
) -> Vec<String> {
    let url = format!(
        "{BASE}/retornaListaCodigoBarasManifesto/{}/{}",
        inicio.format("%Y%m%d"),
        fim.format("%Y%m%d")
    );

    let response = client()
        .post(&url)
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap();

    let status = response.status();
    let text = response.text().await.unwrap_or_default();

    if text.is_empty() || !status.is_success() {
        return Vec::new();
    }

    match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(resp) => {
            let codigos: Vec<String> = resp["codigos"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|c| c.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            if codigos.is_empty() {
                println!(
                    "   Response: {}",
                    serde_json::to_string_pretty(&resp).unwrap_or_default()
                );
            }
            codigos
        }
        Err(_) => Vec::new(),
    }
}

/// Obtém a lista de códigos de barras dos MTRs nos últimos N dias
async fn listar_barcodes(token: &str, dias: i64) -> Vec<String> {
    use chrono::Utc;
    let hoje = Utc::now().date_naive();
    let inicio = hoje - chrono::Duration::days(dias);
    println!("   Período {inicio} a {hoje} ({dias} dias)");
    listar_barcodes_range(token, inicio, hoje).await
}

/// Obtém o manifesto completo pelo código de barras
async fn consultar_mtr_por_barras(token: &str, barcode: &str) -> serde_json::Value {
    client()
        .post(format!("{BASE}/retornaManifesto/{barcode}"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn test_pe_consulta_completa() {
    if !has_env() {
        eprintln!("SKIP: MTR_PE_CNPJ not set");
        return;
    }

    println!("🔐 Autenticando no MTR-PE...");
    let token = get_token().await;
    println!("   Token obtido: {}...", &token[..40]);

    // ─── 1. Listar códigos de barras ─────────────────────────────────────
    println!("\n📋 Buscando MTRs nos últimos 90 dias...");
    let barcodes = listar_barcodes(&token, 90).await;
    println!("   Total de MTRs encontrados: {}", barcodes.len());

    let barcodes = if barcodes.is_empty() {
        // API tem máximo de 30 dias — tenta janelas de 30 dias nos últimos 6 meses
        let mut found: Vec<String> = Vec::new();
        use chrono::Duration;
        let hoje = chrono::Utc::now().date_naive();

        for meses in 0..6 {
            let fim = hoje - Duration::days(meses as i64 * 30);
            let inicio = fim - Duration::days(29);
            if found.is_empty() {
                println!("   Tentando {inicio} a {fim}...");
                let b2 = listar_barcodes_range(&token, inicio, fim).await;
                println!("   Total: {}", b2.len());
                found = b2;
            }
        }

        if found.is_empty() {
            // Último recurso: período de 1 dia (hoje)
            println!("   Tentando apenas hoje...");
            found = listar_barcodes_range(&token, hoje, hoje).await;
            println!("   Total: {}", found.len());
        }

        found
    } else {
        barcodes
    };

    if barcodes.is_empty() {
        eprintln!("⚠️  Não foi possível encontrar MTRs reais para consulta");
        return;
    }

    let exibir = barcodes.iter().take(3).cloned().collect::<Vec<_>>();
    consultar_e_exibir(&token, &exibir).await;
}

async fn consultar_e_exibir(token: &str, barcodes: &[String]) {
    for (i, barcode) in barcodes.iter().enumerate() {
        println!(
            "\n{} Consultando MTR {}: barcode={}",
            "=".repeat(70),
            i + 1,
            barcode
        );

        let mtr = consultar_mtr_por_barras(token, barcode).await;

        // Extrai info básica antes de mostrar JSON completo
        let codigo = mtr["manifestoCodigo"].as_i64().unwrap_or(0);
        let situacao = mtr["situacaoManifestoCodigo"].as_i64().unwrap_or(-1);
        let retorno = mtr["retorno"].as_str().unwrap_or("");
        let retorno_codigo = mtr["retornoCodigo"].as_i64().unwrap_or(-1);

        println!("   manifestoCodigo: {codigo}");
        println!("   situacaoManifestoCodigo: {situacao}");
        println!("   retornoCodigo: {retorno_codigo}");
        println!("   retorno: {retorno}");

        let items = mtr["itemManifestoJSONs"].as_array();
        println!(
            "   itemManifestoJSONs count: {}",
            items.map(|a| a.len()).unwrap_or(0)
        );

        // Mostra JSON completo formatado
        println!(
            "\n{}",
            serde_json::to_string_pretty(&mtr).unwrap_or_default()
        );
        println!("{}", "=".repeat(70));
    }
}
