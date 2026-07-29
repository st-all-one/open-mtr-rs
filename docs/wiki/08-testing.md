# Testes

## Executar

```bash
# Todos os testes
cargo test -p open-mtr-core                       # 96 testes

# Apenas wiremock
cargo test -p open-mtr-core --test wiremock_sinir

# Apenas testes ao vivo (requer .env com token real)
cargo test -p open-mtr-core --test live_sinir_test
```

## Estrutura

```
core/tests/
├── common/
│   └── mod.rs              # Helpers: setup_sinir_server(), create_sinir_client(), json_response()
├── domain_test.rs          # Testes unitários de validação (Cnpj, Cpf, DataMtr, etc.)
├── openmtr_test.rs         # Smoke tests: builder, system, resposta
├── wiremock_sinir.rs       # Consulta MTR com wiremock
├── wiremock_sinir_errors.rs # Erro 404 + download PDF
└── live_sinir_test.rs      # Testes contra API SINIR real (12 testes, pulam sem token)
```

## Testes Unitários (inline)

Validação de newtypes e construção de tipos ficam nos próprios arquivos:

```rust
// core/src/openmtr/domain/newtypes.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_cnpj_valido() { ... }
    #[test]
    fn test_cpf_valido() { ... }
    #[test]
    fn test_data_mtr_parse() { ... }
}
```

## Testes Wiremock

Simulam a API SINIR sem chamadas de rede reais:

```rust
#[tokio::test]
async fn test_sinir_consultar_mtr_com_lista() {
    let server = common::setup_sinir_server().await;
    Mock::given(method("POST"))
        .and(path("/listaManifesto"))
        .respond_with(common::json_response(200, json!({...})))
        .mount(&server).await;

    let client = common::create_sinir_client(&server).await;
    let resp = client.consultar_mtrs(&filtro).await;
    assert!(resp.is_ok());
}
```

## Testes Live

Executam contra a API SINIR real. Pulam automaticamente se `OPENMTR_SINIR_TOKEN` não estiver configurado:

```bash
cp .env.example .env
# edite .env com seu token real
cargo test -p open-mtr-core --test live_sinir_test
```

Cobertura: consulta, filtro por período, tabelas (resíduos, acondicionamentos, classes, etc.), download PDF, token inválido.
