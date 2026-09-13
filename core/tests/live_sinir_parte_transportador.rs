#![allow(clippy::unwrap_used, clippy::pedantic)]

use std::sync::OnceLock;

use open_mtr_core::*;

static ENV_LOADED: OnceLock<()> = OnceLock::new();

fn load_env() {
    ENV_LOADED.get_or_init(|| {
        dotenvy::from_filename_override("../.env").ok();
        dotenvy::from_filename_override(".env").ok();
    });
}

#[tokio::test]
async fn test_sinir_consultar_com_parte_transportador() {
    load_env();
    let token = match std::env::var("SINIR_REFRESH_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            eprintln!("SKIP: sem token SINIR");
            return;
        }
    };

    let client = OpenMtrClient::builder()
        .base(System::Sinir)
        .token(&token)
        .timeout_secs(60)
        .build()
        .await
        .expect("cliente SINIR");

    // Com tipoParceiro Transportador (5) — a empresa é transportadora
    let filtro = FiltroMtr {
        periodo: Some(Periodo::ultimos_dias(30)),
        parte: Some(FiltroParte {
            tipo: TipoParte::Transportador,
            ..Default::default()
        }),
        tamanho: Some(3),
        ..Default::default()
    };
    let resp = client.consultar_mtrs(&filtro).await.expect("consulta");
    println!(
        "Transportador: retornados={} total_api={:?} erros={}",
        resp.dados.len(),
        resp.total,
        resp.erros.len()
    );
    for m in &resp.dados {
        let obs = m.observacao.clone().unwrap_or_default().replace('\n', " ");
        println!("  #{} status={:?} obs={}", m.numero, m.status, obs.chars().take(50).collect::<String>());
    }

    // Sem parte (tipoParceiro=8 Gerador) — deve retornar 0 ou menos
    let filtro2 = FiltroMtr {
        periodo: Some(Periodo::ultimos_dias(30)),
        tamanho: Some(3),
        ..Default::default()
    };
    let resp2 = client.consultar_mtrs(&filtro2).await.expect("consulta 2");
    println!(
        "Sem parte (Gerador=8): retornados={} total_api={:?} erros={}",
        resp2.dados.len(),
        resp2.total,
        resp2.erros.len()
    );

    assert!(!resp.dados.is_empty(), "deveria retornar MTRs como Transportador");
}
