//! Contrato de bytes do *payload enviado* ao MTR-PE.
//!
//! Um campo que desapareça do JSON de recebimento (ex.: `recebimentoMtrCargo`)
//! só se descobre em produção; aqui o corpo exato é fixado com um transporte
//! fake compartilhado, sem rede.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "teste de contrato: falhas devem panicar com a mensagem clara"
)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use jiff::civil::date;
use open_mtr_core::openmtr::error::OpenMTRError;
use open_mtr_core::openmtr::system::System;
use open_mtr_core::openmtr::transport::{HttpRequest, HttpResponse, HttpTransport};
use open_mtr_core::*;

/// Guarda cada requisição enviada e responde com o sucesso do MTR-PE.
#[derive(Default)]
struct Capture {
    sent: Mutex<Vec<HttpRequest>>,
}

struct CapturingTransport(Arc<Capture>);

#[async_trait]
impl HttpTransport for CapturingTransport {
    async fn send(&self, req: HttpRequest) -> Result<HttpResponse, OpenMTRError> {
        self.0.sent.lock().expect("lock").push(req);
        let body = serde_json::to_vec(&serde_json::json!({ "retornoCodigo": 0 })).expect("json");
        let content_length = u64::try_from(body.len()).unwrap_or(u64::MAX);
        Ok(HttpResponse {
            status: 200,
            body,
            content_type: Some("application/json".into()),
            content_length: Some(content_length),
        })
    }
}

#[tokio::test]
async fn receber_mtrs_envia_todos_os_campos_contratuais() {
    let capture = Arc::new(Capture::default());
    let client = OpenMtrClient::builder()
        .base(System::MtrPe)
        .base_url("https://exemplo.test")
        .access_token("token")
        .build_with_transport(CapturingTransport(Arc::clone(&capture)))
        .await
        .expect("cliente");

    let receber = ReceberMtr {
        numero: "MTR-1".into(),
        data_recebimento: date(2026, 6, 1).at(13, 45, 7, 0),
        responsavel: "Fulano".into(),
        observacoes: None,
        motorista: None,
        placa_veiculo: None,
        residuos_recebidos: Vec::new(),
    };
    let resposta = client.receber_mtrs(vec![receber]).await.expect("resposta");
    assert!(resposta.ok);

    let sent = capture.sent.lock().expect("lock");
    let req = sent.last().expect("uma requisição");
    let body: serde_json::Value =
        serde_json::from_slice(req.body.as_deref().expect("corpo")).expect("json do corpo");
    let item = &body["manifestoRecebimentoJSONs"][0];

    assert_eq!(item["manifestoCodigo"], "MTR-1");
    assert_eq!(item["recebimentoMtrResponsavel"], "Fulano");
    // Campo contratual exigido pela API do MTR-PE.
    assert_eq!(item["recebimentoMtrCargo"], "Responsavel");
    assert_eq!(item["recebimentoMtrData"], "20260601");
    assert_eq!(item["transporteMtrData"], "20260601");
    assert!(item["itemManifestoRecebimentoJSONs"].is_array());
}
