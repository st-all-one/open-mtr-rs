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

fn env(key: &str) -> String {
    load_env();
    std::env::var(key).unwrap_or_default()
}

const BASE: &str = "https://mtr.cprh.pe.gov.br/api";

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap()
}

async fn get_token() -> String {
    let unity: u32 = env("MTR_PE_UNITY").parse().unwrap();
    let resp = client()
        .post(format!("{BASE}/gettoken"))
        .json(&serde_json::json!({
            "pessoaCodigo": unity,
            "pessoaCnpj": env("MTR_PE_CNPJ"),
            "usuarioCpf": env("MTR_PE_CPF_USER"),
            "senha": env("MTR_PE_PASSWORD"),
        }))
        .send()
        .await
        .unwrap();
    let json: serde_json::Value = resp.json().await.unwrap();
    json["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_pe_criar_e_receber_mtr() {
    if !has_env() {
        eprintln!("SKIP: MTR_PE_CNPJ not set");
        return;
    }

    let cnpj = env("MTR_PE_CNPJ");
    let unity: u32 = env("MTR_PE_UNITY").parse().unwrap();

    println!("🔐 Autenticando...");
    let token = get_token().await;
    println!("   Token obtido");

    // ─── 1. CRIAR MTR ────────────────────────────────────────────────────
    println!("\n📝 CRIANDO MTR DE TESTE");
    println!("   Gerador:     {cnpj} (unidade {unity})");
    println!("   Transportador: 11201870000145 (unidade 917) — ATR SOLUÇÕES EM RESÍDUOS");
    println!("   Destinador:  {cnpj} (unidade {unity})");
    println!("   Resíduo:     170201 — Madeira (Classe B CONAMA 307/02)");
    println!("   Quantidade:  1.0 tonelada");

    let create_body = serde_json::json!({
        "manifestoJSONDtos": [{
            "cnpGerador": cnpj,
            "codUnidadeGerador": unity,
            "cnpTransportador": "11201870000145",
            "codUnidadeTransportador": 917,
            "cnpDestinador": cnpj,
            "codUnidadeDestinador": unity,
            "seuCodigoReferencia": format!("TEST-OPENMTR-{}", chrono::Utc::now().format("%Y%m%d%H%M%S")),
            "manifGeradorNomeResponsavel": "STALLONE TESTE",
            "manifGeradorCargoResponsavel": "ANALISTA",
            "manifTransportadorNomeMotorista": "MOTORISTA TESTE",
            "manifTransportadorPlacaVeiculo": "ABC-1A34",
            "manifTransportadorDataExpedicao": chrono::Utc::now().format("%Y%m%d").to_string(),
            "manifObservacao": "MTR CRIADO VIA TESTE AUTOMATIZADO OPENMTR - RESIDUO 170201 (MADEIRA)",
            "itemManifestoJSONs": [{
                "quantidade": 1.0,
                "residuo": "170201",
                "codigoAcondicionamento": 2,
                "codigoClasse": 16,
                "codigoTecnologia": 1,
                "codigoTipoEstado": 1,
                "codigoUnidade": 4,
                "tipoDensidadeValor": 1,
                "tipoDensidadeUnidade": 1,
                "manifestoItemObservacao": "TESTE OPENMTR - MADEIRA CLASSE B",
                "manifestoItemCodInterno": "TST-001"
            }]
        }]
    });

    println!("\n📤 Request:");
    println!(
        "{}",
        serde_json::to_string_pretty(&create_body).unwrap_or_default()
    );

    let create_resp = client()
        .post(format!("{BASE}/salvarManifestoLote"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&create_body)
        .send()
        .await
        .unwrap();

    let status = create_resp.status();
    let text = create_resp.text().await.unwrap_or_default();
    println!("\n📥 Response (HTTP {status}):");
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
        println!(
            "{}",
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );

        let dtos = json["manifestoJSONDtos"].as_array();
        if let Some(dtos) = dtos {
            for dto in dtos {
                let codigo = dto["manifestoCodigo"].as_i64();
                let retorno = dto["retorno"].as_str().unwrap_or("");
                let retorno_codigo = dto["retornoCodigo"].as_i64().unwrap_or(-1);

                println!(
                    "\n   → Código: {:?}, retornoCodigo: {retorno_codigo}",
                    codigo
                );
                println!("   → retorno: {retorno}");

                if retorno_codigo == 0 {
                    if let Some(cod) = codigo {
                        println!("\n✅ MTR CRIADO COM SUCESSO! Nº: {cod}");

                        // ─── 2. TESTAR RECEBIMENTO ────────────────────────
                        println!("\n📥 TESTANDO RECEBIMENTO DO MTR {cod}...");

                        let receive_body = serde_json::json!({
                            "manifestoRecebimentoJSONs": [{
                                "manifestoCodigo": cod.to_string(),
                                "recebimentoMtrResponsavel": "STALLONE TESTE",
                                "recebimentoMtrCargo": "ANALISTA",
                                "recebimentoMtrData": chrono::Utc::now().format("%Y%m%d").to_string(),
                                "transporteMtrData": chrono::Utc::now().format("%Y%m%d").to_string(),
                                "recebimentoMtrObs": "RECEBIMENTO VIA TESTE OPENMTR",
                                "nomeMotorista": "MOTORISTA TESTE",
                                "placaVeiculo": "ABC-1A34",
                                "itemManifestoRecebimentoJSONs": [{
                                    "codigoSequencial": 1,
                                    "qtdRecebida": 1.0,
                                    "residuo": "170201",
                                    "codigoTecnologia": 1,
                                    "codigoTipoEstado": 1
                                }]
                            }]
                        });

                        println!("\n📤 Receive Request:");
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&receive_body).unwrap_or_default()
                        );

                        let receive_resp = client()
                            .post(format!("{BASE}/receberManifestoLote"))
                            .header("Authorization", format!("Bearer {token}"))
                            .json(&receive_body)
                            .send()
                            .await
                            .unwrap();

                        let receive_status = receive_resp.status();
                        let receive_text = receive_resp.text().await.unwrap_or_default();
                        println!("\n📥 Receive Response (HTTP {receive_status}):");

                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&receive_text) {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&json).unwrap_or_default()
                            );
                            let r_dtos = json["manifestoRecebimentoJSONs"].as_array();
                            if let Some(r_dtos) = r_dtos {
                                for r_dto in r_dtos {
                                    let rc = r_dto["retornoCodigo"].as_i64().unwrap_or(-1);
                                    let msg = r_dto["retorno"].as_str().unwrap_or("");
                                    println!("\n   → retornoCodigo: {rc}, mensagem: {msg}");
                                    if rc == 0 {
                                        println!("✅ MTR RECEBIDO COM SUCESSO!");
                                    } else {
                                        println!("⚠️  Recebimento falhou: {msg}");
                                    }
                                }
                            }
                        } else {
                            println!("   (resposta não-JSON) {receive_text}");
                        }
                    }
                } else {
                    println!("❌ Criação falhou: {retorno}");
                }
            }
        }
    } else {
        println!("   (resposta não-JSON) {text}");
    }
}
