use crate::core::errors::MtrError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Cnpj(String);

impl Cnpj {
    pub fn new(raw: impl Into<String>) -> Result<Self, MtrError> {
        let s = raw.into();
        let digits: Vec<u32> = s
            .chars()
            .filter(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).expect("ASCII dígito"))
            .collect();

        if digits.len() != 14 {
            return Err(MtrError::validation(
                "CNPJ deve ter 14 dígitos numéricos",
            ));
        }

        if digits.windows(2).all(|w| w[0] == w[1]) {
            return Err(MtrError::validation("CNPJ com dígitos repetidos"));
        }

        let fatores1 = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
        let soma1: u32 = fatores1.iter().zip(&digits).map(|(f, d)| f * d).sum();
        let resto1 = soma1 % 11;
        let dig1 = if resto1 < 2 { 0 } else { 11 - resto1 };
        if digits[12] != dig1 {
            return Err(MtrError::validation(
                "CNPJ inválido: 1º dígito verificador não confere",
            ));
        }

        let fatores2 = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
        let soma2: u32 = fatores2.iter().zip(&digits).map(|(f, d)| f * d).sum();
        let resto2 = soma2 % 11;
        let dig2 = if resto2 < 2 { 0 } else { 11 - resto2 };
        if digits[13] != dig2 {
            return Err(MtrError::validation(
                "CNPJ inválido: 2º dígito verificador não confere",
            ));
        }

        let clean: String = digits.iter().map(|d| char::from_digit(*d, 10).unwrap()).collect();
        Ok(Cnpj(clean))
    }

    pub fn parse_unchecked(s: String) -> Self {
        Cnpj(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Cnpj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = &self.0;
        if d.len() == 14 {
            write!(
                f,
                "{}.{}.{}/{}-{}",
                &d[0..2], &d[2..5], &d[5..8], &d[8..12], &d[12..14]
            )
        } else {
            write!(f, "{}", d)
        }
    }
}

impl Serialize for Cnpj {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Cnpj {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(Cnpj::parse_unchecked(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cnpj_valido() {
        assert!(Cnpj::new("00.000.000/0001-91").is_ok());
    }

    #[test]
    fn test_cnpj_digitos_invalidos() {
        assert!(Cnpj::new("00.000.000/0001-00").is_err());
    }

    #[test]
    fn test_cnpj_repetido() {
        assert!(Cnpj::new("11.111.111/1111-11").is_err());
    }

    #[test]
    fn test_cnpj_tamanho_invalido() {
        assert!(Cnpj::new("123").is_err());
    }

    #[test]
    fn test_cnpj_display() {
        let cnpj = Cnpj::new("11222333000181").unwrap();
        assert_eq!(cnpj.to_string(), "11.222.333/0001-81");
    }

    #[test]
    fn test_cnpj_serialize_deserialize() {
        let cnpj = Cnpj::new("11222333000181").unwrap();
        let json = serde_json::to_string(&cnpj).unwrap();
        assert_eq!(json, "\"11222333000181\"");
        let parsed: Cnpj = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.as_str(), "11222333000181");
    }
}
