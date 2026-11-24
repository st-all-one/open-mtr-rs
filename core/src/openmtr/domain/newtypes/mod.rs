//! Tipos básicos do domínio MTR com validação e normalização na construção.
//!
//! Estes newtypes representam valores que cruzam a fronteira das bindings (CLI, HTTP, WASM) e o
//! JSONL/CSV dos dumps. A normalização acontece no construtor; instâncias criadas pelos
//! construtores validados já estão prontas para serialização, comparação e uso como chave
//! (`parse_unchecked` é a exceção explícita para valores opacos de integrações).
mod cnpj;
mod codigos;
mod datas;
mod quantidade;

pub use cnpj::{Cnpj, Cpf, CpfCnpj};
pub use codigos::{CodigoIbama, PlacaVeiculo};
pub use datas::{DataMtr, TimestampMs};
pub use quantidade::Quantidade;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cnpj_valido() {
        let cnpj = Cnpj::new("11222333000181").unwrap();
        assert_eq!(cnpj.as_str(), "11222333000181");
    }

    #[test]
    fn test_cnpj_formatado() {
        let cnpj = Cnpj::new("11.222.333/0001-81").unwrap();
        assert_eq!(cnpj.as_str(), "11222333000181");
    }

    #[test]
    fn test_cnpj_invalido() {
        assert!(Cnpj::new("00000000000000").is_err());
        assert!(Cnpj::new("12345678901234").is_err());
    }

    #[test]
    fn test_cnpj_display() {
        let cnpj = Cnpj::new("11222333000181").unwrap();
        assert_eq!(cnpj.to_string(), "11.222.333/0001-81");
    }

    #[test]
    fn test_cnpj_serialize() {
        let cnpj = Cnpj::new("11222333000181").unwrap();
        let json = serde_json::to_string(&cnpj).unwrap();
        assert_eq!(json, "\"11.222.333/0001-81\"");
    }

    #[test]
    fn test_cnpj_deserialize() {
        let cnpj: Cnpj = serde_json::from_str("\"11.222.333/0001-81\"").unwrap();
        assert_eq!(cnpj.as_str(), "11222333000181");
    }

    #[test]
    fn test_cpf_valido() {
        let cpf = Cpf::new("52998224725").unwrap();
        assert_eq!(cpf.as_str(), "52998224725");
    }

    #[test]
    fn test_cpf_formatado() {
        let cpf = Cpf::new("529.982.247-25").unwrap();
        assert_eq!(cpf.as_str(), "52998224725");
    }

    #[test]
    fn test_cpf_invalido() {
        assert!(Cpf::new("00000000000").is_err());
        assert!(Cpf::new("12345678901").is_err());
    }

    #[test]
    fn test_cpf_display() {
        let cpf = Cpf::new("52998224725").unwrap();
        assert_eq!(cpf.to_string(), "529.982.247-25");
    }

    #[test]
    fn test_cpf_cnpj_auto_cpf() {
        let doc = CpfCnpj::new("52998224725").unwrap();
        assert!(matches!(doc, CpfCnpj::Cpf(_)));
    }

    #[test]
    fn test_cpf_cnpj_auto_cnpj() {
        let doc = CpfCnpj::new("11222333000181").unwrap();
        assert!(matches!(doc, CpfCnpj::Cnpj(_)));
    }

    #[test]
    fn test_cpf_cnpj_tamanho_invalido() {
        assert!(CpfCnpj::new("1234567890").is_err());
        assert!(CpfCnpj::new("123456789012345").is_err());
    }

    #[test]
    fn test_data_mtr_parse() {
        let data = DataMtr::parse("15-03-2025").unwrap();
        assert_eq!(data.0, jiff::civil::date(2025, 3, 15));
    }

    #[test]
    fn test_data_mtr_display() {
        let data = DataMtr(jiff::civil::date(2025, 3, 15));
        assert_eq!(data.to_string(), "15-03-2025");
    }

    #[test]
    fn test_data_mtr_parse_invalida() {
        assert!(DataMtr::parse("2025-03-15").is_err());
        assert!(DataMtr::parse("15/03/2025").is_err());
    }

    #[test]
    fn test_data_mtr_serialize() {
        let data = DataMtr(jiff::civil::date(2025, 3, 15));
        let json = serde_json::to_string(&data).unwrap();
        assert_eq!(json, "\"15-03-2025\"");
    }

    #[test]
    fn test_timestamp_ms_now() {
        let ts = TimestampMs::now();
        assert!(ts.0 > 1_700_000_000_000);
    }

    #[test]
    fn test_timestamp_ms_serialize() {
        let ts = TimestampMs(1_700_000_000_000);
        let json = serde_json::to_string(&ts).unwrap();
        assert_eq!(json, "1700000000000");
    }

    #[test]
    fn test_timestamp_ms_deserialize() {
        let ts: TimestampMs = serde_json::from_str("1700000000000").unwrap();
        assert_eq!(ts.0, 1_700_000_000_000);
    }
}
