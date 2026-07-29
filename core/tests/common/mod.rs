use open_mtr_core::*;
use wiremock::{MockServer, Mock, ResponseTemplate, matchers::{method, path}};

pub async fn setup_sinir_server() -> MockServer {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200)
            .insert_header("Content-Type", "application/json")
            .set_body_string(r#"{"erro":false,"objetoResposta":"Bearer test-jwt-token","totalRecords":0,"mensagem":null}"#))
        .mount(&server)
        .await;

    server
}

pub async fn create_sinir_client(server: &MockServer) -> open_mtr_core::OpenMtrClient {
    open_mtr_core::OpenMtrClient::builder()
        .base(open_mtr_core::System::Sinir)
        .base_url(server.uri())
        .token("test-integration-token")
        .build()
        .await
        .expect("Failed to create OpenMtrClient")
}

pub fn json_response(status: u16, body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(status)
        .insert_header("Content-Type", "application/json")
        .set_body_string(body.to_string())
}
