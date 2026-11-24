//! Manifestos do MTR-PE sem CDF emitido (detalhe parcial).

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
    reason = "suíte de fixture: atalhos de teste não fazem parte da API de produção"
)]

use jiff::civil::date;
use open_mtr_core::*;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

const BARCODE: &str = "2114180796000031270000091700000521";

/// Payload real de `/retornaManifesto` de um manifesto `Recebido` cujo
/// destinador ainda não emitiu o CDF: `retornoCodigo = 903`
/// ("Erro não identificado"), cabeçalho completo e `itemManifestoJSONs` vazio.
fn manifesto_sem_cdf() -> serde_json::Value {
    serde_json::json!({
        "retornoCodigo": 903,
        "retorno": "Erro não identificado",
        "manifestoCodigo": 2114180796,
        "situacaoManifestoCodigo": 3,
        "cdfCodigo": null,
        "itemManifestoJSONs": [],
        "cnpGerador": "20300157005885",
        "codUnidadeGerador": 3127,
        "cnpTransportador": "11201870000145",
        "codUnidadeTransportador": 917,
        "cnpDestinador": "03279285002779",
        "codUnidadeDestinador": 521,
        "manifData": "20261001",
        "manifDataExpedicao": "20261001",
        "manifDataRecebimento": "20261001"
    })
}

fn json_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .insert_header("Content-Type", "application/json")
        .set_body_string(body.to_string())
}

async fn mock_lista_e_detalhe(server: &MockServer, barcodes: &[&str], detalhe: serde_json::Value) {
    // Janela única de 30 dias: 01/10/2026 a 02/10/2026.
    Mock::given(method("POST"))
        .and(path("/retornaListaCodigoBarasManifesto/20261001/20261002"))
        .respond_with(json_response(serde_json::json!({
            "retornoCodigo": 0,
            "retorno": "Sucesso.",
            "codigos": barcodes
        })))
        .mount(server)
        .await;

    for barcode in barcodes {
        Mock::given(method("POST"))
            .and(path(format!("/retornaManifesto/{barcode}")))
            .respond_with(json_response(detalhe.clone()))
            .mount(server)
            .await;
    }
}

async fn cliente(server: &MockServer) -> OpenMtrClient {
    OpenMtrClient::builder()
        .base(System::MtrPe)
        .base_url(server.uri())
        .token("jwt-de-teste")
        .build()
        .await
        .expect("client mtr-pe")
}

fn filtro_outubro() -> FiltroMtr {
    let inicio = date(2026, 10, 1);
    let fim = date(2026, 10, 2);
    FiltroMtr {
        periodo: Some(Periodo::new(DataMtr::new(inicio), DataMtr::new(fim)).unwrap()),
        ..Default::default()
    }
}

#[tokio::test]
async fn manifesto_com_retorno_903_entra_no_lote_sem_residuos() {
    let server = MockServer::start().await;
    mock_lista_e_detalhe(&server, &[BARCODE], manifesto_sem_cdf()).await;

    let resp = cliente(&server)
        .await
        .consultar_mtrs(&filtro_outubro())
        .await
        .expect("consultar por período");

    assert!(resp.erros.is_empty(), "erros inesperados: {:?}", resp.erros);
    assert_eq!(resp.dados.len(), 1, "dados: {:?}", resp.dados);
    let mtr = &resp.dados[0];
    assert_eq!(mtr.numero, "2114180796");
    assert!(
        matches!(mtr.status, StatusMtr::Recebido),
        "{:?}",
        mtr.status
    );
    assert!(mtr.residuos.is_empty(), "resíduos: {:?}", mtr.residuos);
    assert_eq!(mtr.barcode, BARCODE);
}

#[tokio::test]
async fn resposta_sem_manifesto_codigo_continua_erro() {
    let server = MockServer::start().await;
    mock_lista_e_detalhe(
        &server,
        &[BARCODE],
        serde_json::json!({
            "retornoCodigo": 901,
            "retorno": "Usuário não tem permissão de acesso ao manifesto"
        }),
    )
    .await;

    let resp = cliente(&server)
        .await
        .consultar_mtrs(&filtro_outubro())
        .await
        .expect("consultar por período");

    assert!(resp.dados.is_empty(), "dados: {:?}", resp.dados);
    assert_eq!(resp.erros.len(), 1, "erros: {:?}", resp.erros);
}
