# Testing

## Estrutura

```
core/src/schemas/
├── auth.rs                # #[cfg(test)] — TokenResponse deserialize
├── cnpj.rs                # #[cfg(test)] — Cnpj::new, Display, serde
├── cpf.rs                 # #[cfg(test)] — Cpf::new, CpfCnpj::new
├── manifesto.rs           # #[cfg(test)] — TipoParceiro, CancelaManifestoDTO
├── recebimento.rs         # #[cfg(test)] — TimestampMs, DTOs
├── cdf.rs                 # #[cfg(test)] — FormatoDownload, validar_emissao
├── logistica_reversa.rs   # #[cfg(test)] — DTOs, RastreioEvento
├── integracao_estadual.rs # #[cfg(test)] — ParteMTR, ApiwsManifestoDTO
└── tabelas.rs             # #[cfg(test)] — LazyLock maps, TipoAcondicionamento

core/tests/
└── integration_test.rs    # wiremock: simula API SINIR
```

## Tests Unitários (inline)

Testes de serialização, deserialização e validação vivem em `#[cfg(test)] mod tests`
dentro de cada arquivo de schema. Zero mocking, testam apenas o tipo.

Exemplo:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cnpj_valido() {
        assert!(Cnpj::new("00.000.000/0001-91").is_ok());
    }

    #[test]
    fn test_cnpj_display() {
        let cnpj = Cnpj::new("11222333000181").unwrap();
        assert_eq!(cnpj.to_string(), "11.222.333/0001-81");
    }
}
```

## Tests de Integração (wiremock)

Testes assíncronos com `wiremock` 0.6 simulando a API SINIR.
Em `core/tests/integration_test.rs`:

```rust
async fn setup() -> (MockServer, ManifestoService) {
    let server = MockServer::start().await;
    let config = MtrConfig {
        base_url: server.uri(),
        integration_token: "test-token".into(),
        ..Default::default()
    };
    let client = MtrClient::new(config).unwrap();
    let service = ManifestoService::new(client);

    // Mock do endpoint /token (chamado automaticamente)
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(json_response(200, json!({
            "erro": false,
            "objetoResposta": "Bearer token",
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
        .respond_with(json_response(200, json!({...})))
        .mount(&server).await;

    let result = service.listar(di, df, tipo, 0, 10).await;
    assert!(result.is_ok());
}
```

## Rodando

```bash
cargo test                          # todos (40 unit + 2 integration)
cargo test --test integration_test  # só wiremock
cargo clippy                        # lints (must be 0 warnings)
```
