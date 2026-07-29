#![allow(clippy::unwrap_used, clippy::pedantic)]

use mtr_sinir_core::client::http::MtrClient;
use mtr_sinir_core::core::config::MtrConfig;
use mtr_sinir_core::core::errors::ErrorCategory;
use mtr_sinir_core::schemas::common::DataMtr;
use mtr_sinir_core::schemas::manifesto::TipoParceiro;
use mtr_sinir_core::services::manifesto::ManifestoService;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn json_response(status: u16, body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(status)
        .insert_header("Content-Type", "application/json")
        .set_body_string(body.to_string())
}

async fn setup() -> (MockServer, ManifestoService) {
    let server = MockServer::start().await;
    let config = MtrConfig {
        base_url: server.uri(),
        integration_token: "test-token".into(),
        ..Default::default()
    };
    let client = MtrClient::new(config).unwrap();
    let service = ManifestoService::new(client);

    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(json_response(200, serde_json::json!({
            "erro": false,
            "objetoResposta": "Bearer access-token-jwt",
            "totalRecords": 0,
            "mensagem": null
        })))
        .mount(&server)
        .await;

    (server, service)
}

#[tokio::test]
async fn test_manifesto_listar_ok() {
    let (server, service) = setup().await;

    Mock::given(method("POST"))
        .and(path("/listaManifesto"))
        .respond_with(json_response(200, serde_json::json!({
            "erro": false,
            "objetoResposta": [{"manNumero": "00001", "manStatus": "ATIVO"}],
            "totalRecords": 1,
            "mensagem": null
        })))
        .mount(&server)
        .await;

    let result = service
        .listar(
            DataMtr::new(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            DataMtr::new(chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()),
            TipoParceiro::Gerador,
            0,
            10,
        )
        .await;
    assert!(result.is_ok(), "listar falhou: {:?}", result.err());
    let (lista, total) = result.unwrap();
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0].man_numero, "00001");
    assert_eq!(total, Some(1));
}

#[tokio::test]
async fn test_manifesto_404_error() {
    let (server, service) = setup().await;

    Mock::given(method("POST"))
        .and(path("/listaManifesto"))
        .respond_with(json_response(404, serde_json::json!({
            "erro": true,
            "mensagem": "Recurso não encontrado"
        })))
        .mount(&server)
        .await;

    let result = service
        .listar(
            DataMtr::new(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            DataMtr::new(chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()),
            TipoParceiro::Gerador,
            0,
            10,
        )
        .await;
    assert!(result.is_err());
}
