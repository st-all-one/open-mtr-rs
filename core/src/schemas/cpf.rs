use crate::core::errors::MtrError;
use crate::schemas::cnpj::Cnpj;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Cpf(String);

impl Cpf {
    pub fn new(raw: impl Into<String>) -> Result<Self, MtrError> {
        let s = raw.into();
        let digits: Vec<u32> = s
            .chars()
            .filter(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).expect("ASCII dígito"))
            .collect();

        if digits.len() != 11 {
            return Err(MtrError::validation("CPF deve ter 11 dígitos numéricos"));
        }

        if digits.windows(2).all(|w| w[0] == w[1]) {
            return Err(MtrError::validation("CPF com dígitos repetidos"));
        }

        let fatores1 = [10, 9, 8, 7, 6, 5, 4, 3, 2];
        let soma1: u32 = fatores1.iter().zip(&digits).map(|(f, d)| f * d).sum();
        let resto1 = soma1 % 11;
        let dig1 = if resto1 < 2 { 0 } else { 11 - resto1 };
        if digits[9] != dig1 {
            return Err(MtrError::validation("CPF inválido: 1º dígito verificador não confere"));
        }

        let fatores2 = [11, 10, 9, 8, 7, 6, 5, 4, 3, 2];
        let soma2: u32 = fatores2.iter().zip(&digits).map(|(f, d)| f * d).sum();
        let resto2 = soma2 % 11;
        let dig2 = if resto2 < 2 { 0 } else { 11 - resto2 };
        if digits[10] != dig2 {
            return Err(MtrError::validation("CPF inválido: 2º dígito verificador não confere"));
        }

        let clean: String = digits.iter().map(|d| char::from_digit(*d, 10).unwrap()).collect();
        Ok(Cpf(clean))
    }

    fn parse_unchecked(s: String) -> Self {
        Cpf(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Cpf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = &self.0;
        if d.len() == 11 {
            write!(f, "{}.{}.{}-{}", &d[0..3], &d[3..6], &d[6..9], &d[9..11])
        } else {
            write!(f, "{}", d)
        }
    }
}

impl Serialize for Cpf {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Cpf {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(Cpf::parse_unchecked(s))
    }
}

#[derive(Debug, Clone)]
pub enum CpfCnpj {
    Cpf(Cpf),
    Cnpj(Cnpj),
}

impl CpfCnpj {
    pub fn as_str(&self) -> &str {
        match self {
            CpfCnpj::Cpf(c) => c.as_str(),
            CpfCnpj::Cnpj(c) => c.as_str(),
        }
    }

    pub fn new(raw: impl Into<String>) -> Result<Self, MtrError> {
        let s: String = raw.into();
        let digits: Vec<u32> = s
            .chars()
            .filter(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).expect("ASCII dígito"))
            .collect();
        match digits.len() {
            11 => Cpf::new(s).map(CpfCnpj::Cpf),
            14 => Cnpj::new(s).map(CpfCnpj::Cnpj),
            _ => Err(MtrError::validation(
                "Documento deve ter 11 (CPF) ou 14 (CNPJ) dígitos numéricos",
            )),
        }
    }
}

impl Serialize for CpfCnpj {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            CpfCnpj::Cpf(c) => c.serialize(s),
            CpfCnpj::Cnpj(c) => c.serialize(s),
        }
    }
}

impl<'de> Deserialize<'de> for CpfCnpj {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        let digits: Vec<u32> = s
            .chars()
            .filter(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).expect("ASCII dígito"))
            .collect();
        match digits.len() {
            11 => Ok(CpfCnpj::Cpf(Cpf::parse_unchecked(s))),
            14 => Ok(CpfCnpj::Cnpj(Cnpj::parse_unchecked(s))),
            _ => Err(serde::de::Error::custom(
                "Documento deve ter 11 (CPF) ou 14 (CNPJ) dígitos",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpf_valido() {
        assert!(Cpf::new("529.982.247-25").is_ok());
    }

    #[test]
    fn test_cpf_invalido() {
        assert!(Cpf::new("111.111.111-11").is_err());
    }

    #[test]
    fn test_cpf_cnpj_parse() {
        let cnpj = CpfCnpj::new("11222333000181").unwrap();
        assert!(matches!(cnpj, CpfCnpj::Cnpj(_)));
        let cpf = CpfCnpj::new("529.982.247-25").unwrap();
        assert!(matches!(cpf, CpfCnpj::Cpf(_)));
    }
}
