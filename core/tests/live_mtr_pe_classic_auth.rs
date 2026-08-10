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

fn env(key: &str) -> Option<String> {
    load_env();
    std::env::var(key).ok()
}

fn has_env() -> bool {
    load_env();
    std::env::var("MTR_PE_CNPJ").is_ok()
}

/// Testa o OpenMtrClassicAuth: obtém o token via CPF+senha no MTR-PE real
/// usando o atalho `obter_token` e o builder `OpenMtrClassicAuth::new(...)`.
#[tokio::test]
async fn test_mtr_pe_classic_auth_obter_token() {
    if !has_env() {
        eprintln!("SKIP: MTR_PE_CNPJ not set");
        return;
    }

    let cnpj = env("MTR_PE_CNPJ").unwrap();
    let cpf = env("MTR_PE_CPF_USER").unwrap();
    let password = env("MTR_PE_PASSWORD").unwrap();
    let unity: u32 = env("MTR_PE_UNITY").unwrap().parse().unwrap();

    let params = AuthParams::CpfSenhaUnidade {
        cpf: cpf.clone(),
        senha: password.clone(),
        unidade: unity,
        cnpj: Some(cnpj.clone()),
    };

    // 1. Atalho funcional obter_token
    let token = OpenMtrClassicAuth::obter_token(System::MtrPe, &params)
        .await
        .expect("obter_token MTR-PE");
    assert!(!token.is_empty(), "token não deve ser vazio");
    println!("  ✅ obter_token: {}...", &token[..20]);

    // 2. Builder explícito
    let token2 = OpenMtrClassicAuth::new(System::MtrPe)
        .params(AuthParams::CpfSenhaUnidade {
            cpf,
            senha: password,
            unidade: unity,
            cnpj: Some(cnpj),
        })
        .timeout_secs(60)
        .build()
        .await
        .expect("builder MTR-PE");
    assert!(!token2.is_empty());
    println!("  ✅ builder: {}...", &token2[..20]);

    // 3. Usa o token obtido para iniciar o cliente
    let client = OpenMtrClient::builder()
        .base(System::MtrPe)
        .token(&token)
        .timeout_secs(60)
        .build()
        .await
        .expect("cliente MTR-PE com token clássico");
    println!("  ✅ cliente iniciado com token clássico");
    assert_eq!(client.system_name(), "MTR Pernambuco");
}

/// Testa a serialização do AuthParams (formato esperado nos bindings).
#[test]
fn test_auth_params_json_schema() {
    let params = AuthParams::CpfSenhaUnidade {
        cpf: "07800211428".into(),
        senha: "secret".into(),
        unidade: 3615,
        cnpj: Some("11201870000226".into()),
    };
    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["tipo"], "cpf_senha_unidade");
    assert_eq!(json["unidade"], 3615);
    assert_eq!(json["cnpj"], "11201870000226");
    println!("  ✅ schema: {}", serde_json::to_string(&params).unwrap());
}
