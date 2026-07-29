use open_mtr_core::*;

#[tokio::test]
async fn test_builder_missing_system() {
    let result = OpenMtrClient::builder()
        .token("test")
        .build()
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_builder_missing_token() {
    let result = OpenMtrClient::builder()
        .base(System::Sinir)
        .build()
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_system_default_base_url() {
    assert!(System::Sinir.default_base_url().contains("sinir"));
    assert!(System::Sigor.default_base_url().contains("sigor"));
}

#[test]
fn test_system_from_str() {
    assert_eq!("sinir".parse::<System>().unwrap(), System::Sinir);
    assert_eq!("MTRPE".parse::<System>().unwrap(), System::MtrPe);
    assert!("invalid".parse::<System>().is_err());
}

#[test]
fn test_resposta_types() {
    let id = uuid::Uuid::now_v7();
    let r = Resposta::<String>::sucesso(id, vec!["ok".into()]);
    assert!(r.ok);
    assert_eq!(r.dados.len(), 1);
    assert!(r.erros.is_empty());

    let r2 = Resposta::<String>::com_erros(id, vec![
        Problema::validation("erro de teste")
    ]);
    assert!(!r2.ok);
    assert!(r2.tem_erros());
}
