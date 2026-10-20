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

use open_mtr_core::*;

#[test]
fn test_cnpj_valid() {
    let cnpj = Cnpj::new("11222333000181").expect("valid CNPJ");
    assert_eq!(cnpj.as_str(), "11222333000181");
}

#[test]
fn test_cnpj_invalid() {
    assert!(Cnpj::new("00000000000000").is_err());
    assert!(Cnpj::new("123").is_err());
}

#[test]
fn test_cpf_valid() {
    let cpf = Cpf::new("12345678909").expect("valid CPF");
    assert_eq!(cpf.as_str(), "12345678909");
}

#[test]
fn test_cpf_invalid() {
    assert!(Cpf::new("00000000000").is_err());
    assert!(Cpf::new("123").is_err());
}

#[test]
fn test_cpf_cnpj_auto_detect() {
    let cpf = CpfCnpj::new("12345678909").expect("CPF");
    assert!(matches!(cpf, CpfCnpj::Cpf(_)));

    let cnpj = CpfCnpj::new("11222333000181").expect("CNPJ");
    assert!(matches!(cnpj, CpfCnpj::Cnpj(_)));
}

#[test]
fn test_data_mtr_parse() {
    let data = DataMtr::parse("01-01-2024").expect("valid date");
    assert_eq!(data.to_string(), "01-01-2024");
}

#[test]
fn test_data_mtr_invalid() {
    assert!(DataMtr::parse("32-01-2024").is_err());
    assert!(DataMtr::parse("not-a-date").is_err());
}

#[test]
fn test_timestamp_ms_now() {
    let ts = TimestampMs::now();
    assert!(ts.0 > 0);
}
