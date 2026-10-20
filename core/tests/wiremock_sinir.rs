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

mod common;

use open_mtr_core::*;
use wiremock::{
    Mock,
    matchers::{method, path},
};

#[tokio::test]
async fn test_sinir_consultar_mtr_por_numero() {
    let server = common::setup_sinir_server().await;

    Mock::given(method("GET"))
        .and(path("/retornaManifesto/123456"))
        .respond_with(common::json_response(200, serde_json::json!({
            "erro": false,
            "objetoResposta": {
                "manNumero": "123456",
                "situacaoManifesto": { "simDescricao": "Salvo" },
                "parceiroGerador": { "parCodigo": 1, "parDescricao": "Gerador Ltda" },
                "parceiroTransportador": { "parCodigo": 2, "parDescricao": "Transportador Ltda" },
                "parceiroDestinador": { "parCodigo": 3, "parDescricao": "Destinador Ltda" }
            }
        })))
        .mount(&server)
        .await;

    let client = common::create_sinir_client(&server).await;

    let resp = client
        .consultar_mtrs(&FiltroMtr {
            numeros: Some(vec!["123456".into()]),
            ..Default::default()
        })
        .await;

    assert!(resp.is_ok(), "consulta falhou: {:?}", resp.err());
    let resposta = resp.unwrap();
    assert!(resposta.ok, "resposta não foi ok: {:?}", resposta.erros);
    assert_eq!(resposta.dados.len(), 1);
    assert_eq!(resposta.dados[0].numero, "123456");
}

#[tokio::test]
async fn test_sinir_consultar_mtr_por_periodo() {
    let server = common::setup_sinir_server().await;

    Mock::given(method("POST"))
        .and(path("/listaManifesto"))
        .respond_with(common::json_response(
            200,
            serde_json::json!({
                "erro": false,
                "objetoResposta": [
                    {
                        "manNumero": "123456",
                        "manStatus": "ATIVO"
                    }
                ],
                "totalRecords": 1,
                "mensagem": null
            }),
        ))
        .mount(&server)
        .await;

    let client = common::create_sinir_client(&server).await;

    let resp = client
        .consultar_mtrs(&FiltroMtr {
            periodo: Some(Periodo::ultimos_dias(30)),
            ..Default::default()
        })
        .await;

    assert!(resp.is_ok(), "consulta falhou: {:?}", resp.err());
    let resposta = resp.unwrap();
    assert!(resposta.ok, "resposta não foi ok: {:?}", resposta.erros);
    assert_eq!(resposta.dados.len(), 1);
    assert_eq!(resposta.dados[0].numero, "123456");
}
