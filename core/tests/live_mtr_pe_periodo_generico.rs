#![allow(clippy::unwrap_used, clippy::pedantic)]

use std::sync::OnceLock;

use chrono::{NaiveDate, Utc};
use open_mtr_core::*;

static ENV_LOADED: OnceLock<()> = OnceLock::new();

fn load_env() {
    ENV_LOADED.get_or_init(|| {
        dotenvy::from_filename_override("../.env").ok();
        dotenvy::from_filename_override(".env").ok();
    });
}

fn has_env() -> bool {
    load_env();
    std::env::var("MTR_PE_CNPJ").is_ok()
}

/// Testa o método GENÉRICO consultar_mtrs com filtro por período no MTR-PE.
/// O adapter deve: listar barcodes em janelas de 30 dias e buscar o detalhe.
#[tokio::test]
async fn test_mtr_pe_consultar_por_periodo_generico() {
    if !has_env() {
        eprintln!("SKIP: MTR_PE_CNPJ not set");
        return;
    }

    let cnpj = std::env::var("MTR_PE_CNPJ").unwrap();
    let cpf = std::env::var("MTR_PE_CPF_USER").unwrap();
    let password = std::env::var("MTR_PE_PASSWORD").unwrap();
    let unity: u32 = std::env::var("MTR_PE_UNITY").unwrap().parse().unwrap();

    let token = OpenMtrAuthBuilder::default()
        .base(System::MtrPe)
        .credenciais(&cpf, &password, unity)
        .cnpj(&cnpj)
        .timeout_secs(60)
        .build()
        .await
        .expect("auth mtr-pe");

    let client = OpenMtrClient::builder()
        .base(System::MtrPe)
        .token(&token)
        .timeout_secs(60)
        .build()
        .await
        .expect("client mtr-pe");

    let hoje = Utc::now().date_naive();
    let inicio = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let fim = NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
    assert!(inicio <= hoje);

    let filtro = FiltroMtr {
        periodo: Some(Periodo::new(DataMtr::new(inicio), DataMtr::new(fim)).unwrap()),
        ..Default::default()
    };

    let resp = client
        .consultar_mtrs(&filtro)
        .await
        .expect("consultar por período");
    println!(
        "Total MTRs no período {} a {}: {} (erros: {})",
        inicio,
        fim,
        resp.dados.len(),
        resp.erros.len()
    );

    for e in &resp.erros {
        println!("  erro: {}", e.detail);
    }

    for m in &resp.dados {
        println!(
            "  #{} status={:?} gerador={:?} transportador={:?} exped={:?}",
            m.numero,
            m.status,
            m.gerador.as_ref().map(|g| g.documento.to_string()),
            m.transportador.as_ref().map(|t| t.documento.to_string()),
            m.data_expedicao,
        );
    }

    assert!(!resp.dados.is_empty(), "deveria encontrar MTRs no período");
}
