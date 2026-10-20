use crate::openmtr::error::OpenMTRError;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cnpj(String);

impl Cnpj {
    #[allow(
        clippy::indexing_slicing,
        reason = "o comprimento 14 e o conteúdo ASCII numérico são validados no início"
    )]
    fn validar_digitos(digits: &str) -> bool {
        if digits.len() != 14 || !digits.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        let Some(first) = digits.chars().next() else {
            return false;
        };
        if digits.chars().all(|c| c == first) {
            return false;
        }
        let nums: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();

        let pesos1: [u32; 12] = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
        let soma1: u32 = nums[..12]
            .iter()
            .zip(pesos1.iter())
            .map(|(n, p)| n * p)
            .sum();
        let dig1 = if soma1 % 11 < 2 { 0 } else { 11 - (soma1 % 11) };
        if dig1 != nums[12] {
            return false;
        }

        let pesos2: [u32; 13] = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
        let soma2: u32 = nums[..13]
            .iter()
            .zip(pesos2.iter())
            .map(|(n, p)| n * p)
            .sum();
        let dig2 = if soma2 % 11 < 2 { 0 } else { 11 - (soma2 % 11) };
        dig2 == nums[13]
    }

    pub fn new(digits: &str) -> Result<Self, OpenMTRError> {
        let clean: String = digits.chars().filter(char::is_ascii_digit).collect();
        if Self::validar_digitos(&clean) {
            Ok(Self(clean))
        } else {
            Err(OpenMTRError::validation(format!("CNPJ inválido: {digits}")))
        }
    }

    #[must_use]
    pub fn parse_unchecked(s: &str) -> Self {
        Self(s.chars().filter(char::is_ascii_digit).collect())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Cnpj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}/{}-{}",
            self.0.get(..2).unwrap_or_default(),
            self.0.get(2..5).unwrap_or_default(),
            self.0.get(5..8).unwrap_or_default(),
            self.0.get(8..12).unwrap_or_default(),
            self.0.get(12..).unwrap_or_default()
        )
    }
}

impl Serialize for Cnpj {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Cnpj {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cpf(String);

impl Cpf {
    #[allow(
        clippy::indexing_slicing,
        reason = "o comprimento 11 e o conteúdo ASCII numérico são validados no início"
    )]
    fn validar_digitos(digits: &str) -> bool {
        if digits.len() != 11 || !digits.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        let Some(first) = digits.chars().next() else {
            return false;
        };
        if digits.chars().all(|c| c == first) {
            return false;
        }
        let nums: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();

        let soma1: u32 = nums[..9]
            .iter()
            .zip((2..=10).rev())
            .map(|(n, p)| n * p)
            .sum();
        let dig1 = if soma1 % 11 < 2 { 0 } else { 11 - (soma1 % 11) };
        if dig1 != nums[9] {
            return false;
        }

        let soma2: u32 = nums[..10]
            .iter()
            .zip((2..=11).rev())
            .map(|(n, p)| n * p)
            .sum();
        let dig2 = if soma2 % 11 < 2 { 0 } else { 11 - (soma2 % 11) };
        dig2 == nums[10]
    }

    pub fn new(digits: &str) -> Result<Self, OpenMTRError> {
        let clean: String = digits.chars().filter(char::is_ascii_digit).collect();
        if Self::validar_digitos(&clean) {
            Ok(Self(clean))
        } else {
            Err(OpenMTRError::validation(format!("CPF inválido: {digits}")))
        }
    }

    #[must_use]
    pub fn parse_unchecked(s: &str) -> Self {
        Self(s.chars().filter(char::is_ascii_digit).collect())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Cpf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}-{}",
            self.0.get(..3).unwrap_or_default(),
            self.0.get(3..6).unwrap_or_default(),
            self.0.get(6..9).unwrap_or_default(),
            self.0.get(9..).unwrap_or_default()
        )
    }
}

impl Serialize for Cpf {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Cpf {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug)]
pub enum CpfCnpj {
    Cpf(Cpf),
    Cnpj(Cnpj),
}

impl CpfCnpj {
    pub fn new(s: &str) -> Result<Self, OpenMTRError> {
        let clean: String = s.chars().filter(char::is_ascii_digit).collect();
        match clean.len() {
            11 => Ok(Self::Cpf(Cpf::new(&clean)?)),
            14 => Ok(Self::Cnpj(Cnpj::new(&clean)?)),
            _ => Err(OpenMTRError::validation(format!(
                "documento deve ter 11 (CPF) ou 14 (CNPJ) dígitos, encontrado {}: {s}",
                clean.len()
            ))),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Cpf(c) => c.as_str(),
            Self::Cnpj(c) => c.as_str(),
        }
    }
}

impl fmt::Display for CpfCnpj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cpf(c) => write!(f, "{c}"),
            Self::Cnpj(c) => write!(f, "{c}"),
        }
    }
}

impl Serialize for CpfCnpj {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Cpf(c) => c.serialize(serializer),
            Self::Cnpj(c) => c.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for CpfCnpj {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug)]
pub struct DataMtr(pub NaiveDate);

impl DataMtr {
    #[must_use]
    pub fn new(d: NaiveDate) -> Self {
        Self(d)
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        NaiveDate::parse_from_str(s, "%d-%m-%Y")
            .map(Self)
            .map_err(|e| format!("data inválida (use dd-MM-yyyy): {e}"))
    }
}

impl fmt::Display for DataMtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%d-%m-%Y"))
    }
}

impl Serialize for DataMtr {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DataMtr {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug)]
pub struct TimestampMs(pub i64);

impl TimestampMs {
    #[must_use]
    pub fn now() -> Self {
        Self(crate::openmtr::time_util::utc_now().timestamp_millis())
    }

    #[must_use]
    pub fn from_datetime(dt: NaiveDateTime) -> Self {
        Self(dt.and_utc().timestamp_millis())
    }
}

impl Serialize for TimestampMs {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TimestampMs {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        i64::deserialize(deserializer).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Quantidade(f64);

impl Quantidade {
    pub fn new(valor: f64) -> Result<Self, OpenMTRError> {
        if valor < 0.0 {
            return Err(OpenMTRError::validation(format!(
                "Quantidade não pode ser negativa: {valor}"
            )));
        }
        if valor > 1_000_000_000.0 {
            return Err(OpenMTRError::validation(format!(
                "Quantidade acima do limite: {valor}"
            )));
        }
        Ok(Self(valor))
    }

    #[allow(
        dead_code,
        reason = "campo/helper mantido para compatibilidade da API e testes"
    )]
    pub(crate) fn new_unchecked(valor: f64) -> Self {
        Self(valor)
    }

    #[must_use]
    pub fn get(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for Quantidade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CodigoIbama(String);

impl CodigoIbama {
    pub fn new(digits: &str) -> Result<Self, OpenMTRError> {
        let cleaned: String = digits.chars().filter(char::is_ascii_digit).collect();
        if cleaned.len() != 6 && cleaned.len() != 8 {
            return Err(OpenMTRError::validation(format!(
                "Código IBAMA deve ter 6 ou 8 dígitos: {digits}"
            )));
        }
        Ok(Self(cleaned))
    }

    #[must_use]
    pub fn parse_unchecked(s: &str) -> Self {
        Self(s.to_string())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CodigoIbama {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacaVeiculo(String);

impl PlacaVeiculo {
    pub fn new(s: &str) -> Result<Self, OpenMTRError> {
        let upper = s.to_uppercase();
        // Aceita formato ABC1D23 (mercosul) ou ABC-1234 (antigo)
        let is_mercosul = upper.len() == 7
            && upper.chars().take(3).all(|c| c.is_ascii_alphabetic())
            && upper.chars().nth(3).is_some_and(|c| c.is_ascii_digit())
            && upper
                .chars()
                .nth(4)
                .is_some_and(|c| c.is_ascii_alphabetic())
            && upper.chars().skip(5).all(|c| c.is_ascii_digit());
        let is_antiga = upper.len() == 8
            && upper.chars().take(3).all(|c| c.is_ascii_alphabetic())
            && upper.chars().nth(3) == Some('-')
            && upper.chars().skip(4).all(|c| c.is_ascii_digit());
        if !is_mercosul && !is_antiga {
            return Err(OpenMTRError::validation(format!(
                "Placa inválida: {s} (use ABC1D23 ou ABC-1234)"
            )));
        }
        Ok(Self(upper))
    }

    #[must_use]
    pub fn parse_unchecked(s: &str) -> Self {
        Self(s.to_uppercase())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PlacaVeiculo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
        assert_eq!(data.0, NaiveDate::from_ymd_opt(2025, 3, 15).unwrap());
    }

    #[test]
    fn test_data_mtr_display() {
        let data = DataMtr(NaiveDate::from_ymd_opt(2025, 3, 15).unwrap());
        assert_eq!(data.to_string(), "15-03-2025");
    }

    #[test]
    fn test_data_mtr_parse_invalida() {
        assert!(DataMtr::parse("2025-03-15").is_err());
        assert!(DataMtr::parse("15/03/2025").is_err());
    }

    #[test]
    fn test_data_mtr_serialize() {
        let data = DataMtr(NaiveDate::from_ymd_opt(2025, 3, 15).unwrap());
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
