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

//! Indexação geral de MTRs do MTR-PE (Pernambuco) transportados por uma empresa
//! transportadora a partir de uma data inicial (padrão 01/01/2026).
//!
//! Usa o método GENÉRICO `consultar_mtrs` do open-mtr-core com filtro por período.
//! O adapter MTR-PE implementa a lógica de listar códigos de barras
//! (janelas de 30 dias) + consulta de detalhe automaticamente.
//!
//! Uso:
//!   cargo run -p open-mtr-core --example indexar_mtr_pe                       # de 01/01/2026 a hoje
//!   cargo run -p open-mtr-core --example indexar_mtr_pe -- --inicio 2026-01-01 --fim 2026-08-10 --saida indice.json
//!
//! Requer o .env na raiz do projeto com:
//!   MTR_PE_CNPJ, MTR_PE_CPF, MTR_PE_PASSWORD, MTR_PE_UNITY

use std::collections::HashSet;
use std::sync::OnceLock;

use chrono::{NaiveDate, Utc};
use open_mtr_core::*;
use serde_json::{Map, Value, json};

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

fn somente_digitos(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn cnpj_raiz(cnpj: &str) -> String {
    somente_digitos(cnpj).chars().take(8).collect()
}

fn status_str(status: &StatusMtr) -> String {
    status.to_string()
}

fn parte_para_json(p: &Option<Parte>) -> Value {
    match p {
        Some(parte) => json!({
            "cnpj": parte.documento.as_str(),
            "unidade": parte.unidade,
        }),
        None => Value::Null,
    }
}

fn mtr_para_json(m: &Mtr) -> Value {
    json!({
        "numero": m.numero.parse::<i64>().ok(),
        "barcode": m.barcode,
        "status": status_str(&m.status),
        "data_criacao": m.data_criacao.map(|d| d.format("%Y%m%d").to_string()),
        "data_expedicao": m.data_expedicao.map(|d| d.format("%Y%m%d").to_string()),
        "data_recebimento": m.data_recebimento.map(|d| d.format("%Y%m%d").to_string()),
        "observacao": m.observacao,
        "motorista": m.motorista,
        "placa": m.placa,
        "gerador": parte_para_json(&m.gerador),
        "transportador": parte_para_json(&m.transportador),
        "destinador": parte_para_json(&m.destinador),
        "itens": m.residuos.iter().map(|r| json!({
            "residuo_ibama": r.codigo_ibama,
            "quantidade": r.quantidade,
            "quantidade_recebida": r.quantidade_recebida,
            "unidade": r.unidade,
            "classe": r.classe,
            "tratamento": r.tratamento,
            "estado_fisico": r.estado_fisico,
            "acondicionamento": r.acondicionamento,
            "codigo_interno": r.codigo_interno,
            "descricao_interna": r.descricao_interna,
        })).collect::<Vec<_>>(),
        "quantidade_itens": m.residuos.len(),
    })
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut inicio = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut fim = Utc::now().date_naive();
    let mut saida = "indice_mtr_pe.json".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--inicio" if i + 1 < args.len() => {
                inicio = NaiveDate::parse_from_str(&args[i + 1], "%Y-%m-%d")
                    .unwrap_or_else(|_| panic!("Data inválida em --inicio: {}", args[i + 1]));
                i += 1;
            }
            "--fim" if i + 1 < args.len() => {
                fim = NaiveDate::parse_from_str(&args[i + 1], "%Y-%m-%d")
                    .unwrap_or_else(|_| panic!("Data inválida em --fim: {}", args[i + 1]));
                i += 1;
            }
            "--saida" if i + 1 < args.len() => {
                saida = args[i + 1].clone();
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    println!("{}", "=".repeat(70));
    println!("  MTR-PE — Indexação geral de MTRs transportados (método genérico)");
    println!("{}", "=".repeat(70));

    let cnpj = env("MTR_PE_CNPJ").expect("MTR_PE_CNPJ no .env");
    let cpf = env("MTR_PE_CPF").expect("MTR_PE_CPF no .env");
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
        .timeout_secs(120)
        .build()
        .await
        .expect("autenticação MTR-PE");
    println!("  ✅ Autenticado no MTR-PE");

    let client = OpenMtrClient::builder()
        .base(System::MtrPe)
        .token(&token)
        .timeout_secs(120)
        .build()
        .await
        .expect("cliente MTR-PE");

    let raiz_empresa = cnpj_raiz(&cnpj);
    println!("  🏢 Empresa CNPJ {cnpj} (raiz {raiz_empresa})");

    let filtro = FiltroMtr {
        periodo: Some(Periodo::new(DataMtr::new(inicio), DataMtr::new(fim)).unwrap()),
        ..Default::default()
    };

    println!("   Consultando MTRs de {inicio} a {fim}...");
    let resp = client
        .consultar_mtrs(&filtro)
        .await
        .expect("consulta por período");
    let total_retornado = resp.dados.len();

    println!(
        "   Retornados pelo adapter: {total_retornado} (erros: {})",
        resp.erros.len()
    );
    for e in &resp.erros {
        println!("   ⚠️  erro: {}", e.detail);
    }

    // Filtra MTRs em que a empresa é o transportador
    let mut mtrs: Vec<Value> = resp
        .dados
        .iter()
        .filter(|m| {
            m.transportador
                .as_ref()
                .is_some_and(|t| cnpj_raiz(t.documento.as_str()) == raiz_empresa)
        })
        .map(mtr_para_json)
        .collect();

    mtrs.sort_by_key(|m| {
        m.get("data_expedicao")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    });

    // Resumo por status
    let mut por_status: Map<String, Value> = Map::new();
    for m in &mtrs {
        let status = m.get("status").and_then(Value::as_str).unwrap_or("?");
        let contagem = por_status
            .entry(status.to_string())
            .or_insert_with(|| json!(0));
        *contagem = json!(contagem.as_i64().unwrap_or(0) + 1);
    }

    // Barcodes únicos (apenas contagem para o resumo)
    let unicos: HashSet<String> = resp
        .dados
        .iter()
        .filter(|m| {
            m.transportador
                .as_ref()
                .is_some_and(|t| cnpj_raiz(t.documento.as_str()) == raiz_empresa)
        })
        .map(|m| m.numero.clone())
        .collect();

    let indice = json!({
        "meta": {
            "sistema": "MTR-PE",
            "metodo": "consultar_mtrs generico (periodo)",
            "base_url": System::MtrPe.default_base_url(),
            "empresa_cnpj": cnpj,
            "empresa_cnpj_raiz": raiz_empresa,
            "unidade": unity,
            "periodo_inicio": inicio.format("%Y-%m-%d").to_string(),
            "periodo_fim": fim.format("%Y-%m-%d").to_string(),
            "gerado_em": Utc::now().to_rfc3339(),
        },
        "resumo": {
            "total_mtrs": mtrs.len(),
            "total_retornados_adapter": total_retornado,
            "mtrs_nao_transportados_descartados": total_retornado - mtrs.len() - resp.erros.len(),
            "erros_consulta": resp.erros.len(),
            "por_status": por_status,
        },
        "erros": resp.erros.iter().map(|e| json!({ "status": e.status, "erro": e.detail })).collect::<Vec<_>>(),
        "mtrs": mtrs,
    });

    std::fs::write(&saida, serde_json::to_string_pretty(&indice).unwrap())
        .unwrap_or_else(|e| panic!("Falha ao salvar JSON: {e}"));
    println!("  💾 Índice salvo em: {saida}");

    println!();
    println!("  📊 RESUMO:");
    println!(
        "     Total de MTRs transportados: {}",
        indice["resumo"]["total_mtrs"]
    );
    println!(
        "     Total de barcodes únicos processados: {}",
        unicos.len()
    );
    for (k, v) in por_status {
        println!("     - {k}: {v}");
    }
    println!();
    println!("  ✅ Concluído!");
}
