use super::*;
use crate::openmtr::domain::FiltroMtr;
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::system::System;
use crate::openmtr::time_util::{Clock, FixedClock, SequentialIdGenerator};
use crate::openmtr::transport::{HttpRequest, HttpResponse, HttpTransport};
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Mutex;
use uuid::Uuid;

/// Transporte fake síncrono: entrega respostas pré-carregadas, em ordem.
/// Sem rede e sem timers, o resultado é bit a bit reprodutível.
struct FakeTransport {
    respostas: Mutex<VecDeque<serde_json::Value>>,
}

impl FakeTransport {
    fn new(respostas: Vec<serde_json::Value>) -> Self {
        Self {
            respostas: Mutex::new(respostas.into()),
        }
    }
}

#[async_trait]
impl HttpTransport for FakeTransport {
    async fn send(&self, _req: HttpRequest) -> Result<HttpResponse, OpenMTRError> {
        let body = self
            .respostas
            .lock()
            .expect("lock do transporte fake")
            .pop_front()
            .expect("resposta fake disponível");
        let bytes = serde_json::to_vec(&body).map_err(OpenMTRError::from)?;
        let content_length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        Ok(HttpResponse {
            status: 200,
            body: bytes,
            content_type: Some("application/json".into()),
            content_length: Some(content_length),
        })
    }
}

fn fixed_clock() -> Arc<FixedClock> {
    Arc::new(FixedClock::new(
        chrono::DateTime::from_timestamp(1_780_000_000, 0).expect("timestamp fixo"),
    ))
}

#[tokio::test]
async fn injected_id_generator_drives_envelope_id() {
    let transport = FakeTransport::new(vec![
        serde_json::json!({"objetoResposta": [], "totalRecords": 0}),
        serde_json::json!({"objetoResposta": [], "totalRecords": 0}),
    ]);
    let client = OpenMtrClient::builder()
        .base(System::Sinir)
        .base_url("https://exemplo.test")
        .access_token("token-estatico")
        .clock(fixed_clock())
        .id_generator(Arc::new(SequentialIdGenerator::default()))
        .build_with_transport(transport)
        .await
        .expect("cliente construído");

    let filtro = FiltroMtr::default();
    let primeira = client.consultar_mtrs(&filtro).await.expect("consulta ok");
    let segunda = client.consultar_mtrs(&filtro).await.expect("consulta ok");

    // SequentialIdGenerator começa em 1: a 1ª e a 2ª consulta recebem 1 e 2.
    assert_eq!(primeira.id, Uuid::from_u128(1));
    assert_eq!(segunda.id, Uuid::from_u128(2));
    assert_ne!(primeira.id, segunda.id);
}

#[tokio::test]
async fn injected_clock_is_the_today_source() {
    let transport = FakeTransport::new(vec![serde_json::json!({
        "objetoResposta": [],
        "totalRecords": 0
    })]);
    let client = OpenMtrClient::builder()
        .base(System::Sinir)
        .base_url("https://exemplo.test")
        .access_token("token-estatico")
        .clock(fixed_clock())
        .id_generator(Arc::new(SequentialIdGenerator::default()))
        .build_with_transport(transport)
        .await
        .expect("cliente construído");

    // Com filtro sem período, nenhuma data atual é decisiva; o valor observado
    // é o próprio relógio injetado. Garantimos que ele é usado, não `Utc::now`.
    assert_eq!(client.ctx.today(), fixed_clock().today());
}
