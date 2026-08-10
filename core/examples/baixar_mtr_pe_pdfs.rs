//! Download dos PDFs dos MTRs do MTR-PE listados no índice gerado por
//! `indexar_mtr_pe.rs`.
//!
//! O índice (`indice_mtr_pe.json`) contém os MTRs transportados pela empresa,
//! com `numero` e `barcode` (código de barras completo de 34 dígitos, fixado
//! no objeto genérico `Mtr` quando a API o fornece).
//!
//! Os PDFs são salvos em `mtrs/pe/{numero}.pdf`.
//!
//! Uso:
//!   cargo run -p open-mtr-core --example baixar_mtr_pe_pdfs
//!   cargo run -p open-mtr-core --example baixar_mtr_pe_pdfs -- --indice indice_mtr_pe.json --dir mtrs/pe
//!
//! Requer o .env na raiz do projeto com:
//!   MTR_PE_CNPJ, MTR_PE_CPF_USER, MTR_PE_PASSWORD, MTR_PE_UNITY

#![allow(clippy::unwrap_used, clippy::pedantic)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use open_mtr_core::*;
use serde_json::Value;

static ENV_LOADED: OnceLock<()> = OnceLock::new();

fn load_env() {
    ENV_LOADED.get_or_init(|| {
        dotenvy::from_filename_override("../.env").ok();
        dotenvy::from_filename_override(".env").ok();
    });
}

fn env(key: &str) -> Option<String> {
    load_env();
    std::env::var(key).ok()
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut indice_path = "indice_mtr_pe.json".to_string();
    let mut dir = "mtrs/pe".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--indice" if i + 1 < args.len() => {
                indice_path = args[i + 1].clone();
                i += 1;
            }
            "--dir" if i + 1 < args.len() => {
                dir = args[i + 1].clone();
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    println!("{}", "=".repeat(70));
    println!("  MTR-PE — Download de PDFs dos MTRs transportados");
    println!("{}", "=".repeat(70));

    let indice: Value = serde_json::from_str(
        &std::fs::read_to_string(&indice_path)
            .unwrap_or_else(|e| panic!("Falha ao ler índice {indice_path}: {e}")),
    )
    .expect("JSON do índice inválido");

    let mtrs = indice
        .get("mtrs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    println!("   MTRs no índice: {}", mtrs.len());

    // Mapa barcode -> numero, usando o barcode fixado no objeto genérico
    let mut barcode_para_numero: HashMap<String, String> = HashMap::new();
    let mut sem_barcode = 0;
    for m in &mtrs {
        let numero = m
            .get("numero")
            .and_then(Value::as_i64)
            .map(|n| n.to_string())
            .unwrap_or_default();
        let barcode = m.get("barcode").and_then(Value::as_str).unwrap_or("");
        if barcode.is_empty() {
            sem_barcode += 1;
            println!("   ⚠️  MTR {numero}: sem barcode no índice");
        } else {
            barcode_para_numero.insert(barcode.to_string(), numero);
        }
    }
    println!(
        "   Barcodes disponíveis: {} (sem: {sem_barcode})",
        barcode_para_numero.len()
    );

    if barcode_para_numero.is_empty() {
        std::process::exit(1);
    }

    // Autenticação via biblioteca
    let cnpj = env("MTR_PE_CNPJ").expect("MTR_PE_CNPJ no .env");
    let cpf = env("MTR_PE_CPF_USER").expect("MTR_PE_CPF_USER no .env");
    let password = env("MTR_PE_PASSWORD").expect("MTR_PE_PASSWORD no .env");
    let unity: u32 = env("MTR_PE_UNITY")
        .expect("MTR_PE_UNITY no .env")
        .parse()
        .expect("MTR_PE_UNITY numérico");

    let token = OpenMtrClassicAuth::new(System::MtrPe)
        .params(AuthParams::CpfSenhaUnidade {
            cpf: cpf.clone(),
            senha: password.clone(),
            unidade: unity,
            cnpj: Some(cnpj.clone()),
        })
        .timeout_secs(180)
        .build()
        .await
        .expect("autenticação MTR-PE");
    println!("  ✅ Autenticado no MTR-PE");

    let client = OpenMtrClient::builder()
        .base(System::MtrPe)
        .token(&token)
        .timeout_secs(180)
        .build()
        .await
        .expect("cliente MTR-PE");

    // Download em lotes para não estourar o servidor
    let dir = PathBuf::from(&dir);
    std::fs::create_dir_all(&dir).expect("criar diretório mtrs/pe");

    let barcodes: Vec<(String, String)> = barcode_para_numero
        .iter()
        .map(|(b, n)| (b.clone(), n.clone()))
        .collect();
    let total = barcodes.len();
    let mut ok = 0;
    let mut falhas = 0;
    const LOTE: usize = 20;

    for (idx, chunk) in barcodes.chunks(LOTE).enumerate() {
        let nums: Vec<String> = chunk.iter().map(|(b, _)| b.clone()).collect();

        match client.download_mtrs(nums).await {
            Ok(resp) => {
                for item in resp.dados {
                    // item.filename = "{barcode}.pdf"
                    let barcode = item.filename.trim_end_matches(".pdf");
                    let numero = barcode_para_numero.get(barcode).cloned().unwrap_or_default();
                    let caminho = dir.join(format!("{numero}.pdf"));
                    match std::fs::write(&caminho, &item.buffer) {
                        Ok(_) => {
                            ok += 1;
                            println!("   ✅ {numero}.pdf ({} bytes)", item.buffer.len());
                        }
                        Err(e) => {
                            falhas += 1;
                            println!("   ❌ {numero}.pdf: erro ao salvar: {e}");
                        }
                    }
                }
                for erro in &resp.erros {
                    falhas += 1;
                    println!("   ❌ erro no lote {}: {}", idx + 1, erro.detail);
                }
            }
            Err(e) => {
                falhas += chunk.len();
                println!("   ❌ lote {} falhou: {e}", idx + 1);
            }
        }
    }

    println!();
    println!("  📊 RESUMO:");
    println!("     Total: {total}");
    println!("     Baixados com sucesso: {ok}");
    println!("     Falhas: {falhas}");
    println!("     Diretório: {}", dir.display());
    println!();
    if falhas == 0 {
        println!("  ✅ Todos os PDFs baixados!");
    } else {
        println!("  ⚠️  {} download(s) com falha.", falhas);
    }
}
