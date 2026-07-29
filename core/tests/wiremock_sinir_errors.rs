mod common;

use open_mtr_core::*;
use wiremock::{Mock, matchers::{method, path}, ResponseTemplate};

#[tokio::test]
async fn test_sinir_erro_404() {
    let server = common::setup_sinir_server().await;

    Mock::given(method("GET"))
        .and(path("/retornaManifesto/999999"))
        .respond_with(common::json_response(404, serde_json::json!({
            "erro": true,
            "mensagem": "Recurso não encontrado"
        })))
        .mount(&server)
        .await;

    let client = common::create_sinir_client(&server).await;

    let resp = client.consultar_mtrs(&FiltroMtr {
        numeros: Some(vec!["999999".into()]),
        ..Default::default()
    }).await;

    assert!(resp.is_ok(), "esperava Ok com erro de negócio, recebeu Err");
    let resposta = resp.unwrap();
    assert!(!resposta.ok || resposta.tem_erros(), "resposta deveria indicar erro");
}

#[tokio::test]
async fn test_sinir_download_mtr_pdf() {
    let server = common::setup_sinir_server().await;
    let pdf_bytes = vec![0x25, 0x50, 0x44, 0x46, 0x2d]; // %PDF-

    Mock::given(method("POST"))
        .and(path("/downloadManifesto/123456"))
        .respond_with(ResponseTemplate::new(200)
            .insert_header("Content-Type", "application/pdf")
            .set_body_bytes(pdf_bytes.clone()))
        .mount(&server)
        .await;

    let client = common::create_sinir_client(&server).await;

    let resp = client.download_mtrs(vec!["123456".into()]).await;
    assert!(resp.is_ok(), "download falhou: {:?}", resp.err());
    let resposta = resp.unwrap();
    assert_eq!(resposta.dados.len(), 1);
    assert_eq!(resposta.dados[0].filename, "123456.pdf");
    assert!(!resposta.dados[0].buffer.is_empty(), "buffer vazio");
}
