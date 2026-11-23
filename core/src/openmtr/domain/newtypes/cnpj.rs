use crate::openmtr::error::OpenMTRError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// CNPJ de pessoa jurídica normalizado como os 14 dígitos significativos.
///
/// Aceita entrada pontuada (`11.222.333/0001-81`) ou apenas dígitos; [`Self::new`] descarta
/// não dígitos ASCII, rejeita sequências repetidas e confere os dois dígitos verificadores
/// (módulo 11). Serializa como string pontuada `11.222.333/0001-81`.
/// Identifica pessoas jurídicas (gerador, transportador, destinador) no MTR.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cnpj(String);

impl Cnpj {
    /// Verifica os 14 dígitos ASCII e os dois dígitos verificadores do CNPJ.
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

    /// Normaliza e valida o CNPJ, removendo não dígitos e conferindo os verificadores.
    ///
    /// Falha com [`crate::openmtr::error::OpenMTRError`] de validação se o resultado não tiver
    /// 14 dígitos ou se os verificadores não conferirem. O valor guardado é sempre nu.
    pub fn new(digits: &str) -> Result<Self, OpenMTRError> {
        let clean: String = digits.chars().filter(char::is_ascii_digit).collect();
        if Self::validar_digitos(&clean) {
            Ok(Self(clean))
        } else {
            Err(OpenMTRError::validation(format!("CNPJ inválido: {digits}")))
        }
    }

    /// Normaliza um CNPJ sem validar os dígitos verificadores.
    ///
    /// Preserva apenas os dígitos ASCII informados; útil para valores opacos de integrações
    /// (por exemplo o CNPJ curinga `00000000000000`).
    #[must_use]
    pub fn parse_unchecked(s: &str) -> Self {
        Self(s.chars().filter(char::is_ascii_digit).collect())
    }

    /// Os 14 dígitos do CNPJ, sem pontuação.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Escreve o CNPJ no formato canônico `NN.NNN.NNN/NNNN-NN`.
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

/// Serializa o CNPJ como string pontuada, igual à saída de [`fmt::Display`].
impl Serialize for Cnpj {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

/// Desserializa um CNPJ de string pontuada ou somente dígitos, validando-o.
impl<'de> Deserialize<'de> for Cnpj {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}

/// CPF de pessoa física normalizado como os 11 dígitos significativos.
///
/// Aceita entrada pontuada (`529.982.247-25`) ou apenas dígitos; [`Self::new`] descarta
/// não dígitos ASCII, rejeita sequências repetidas e confere os dois dígitos verificadores
/// (módulo 11). Serializa como string pontuada `529.982.247-25`.
/// Identifica pessoas físicas envolvidas no MTR.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cpf(String);

impl Cpf {
    /// Verifica os 11 dígitos ASCII e os dois dígitos verificadores do CPF.
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

    /// Normaliza e valida o CPF, removendo não dígitos e conferindo os verificadores.
    ///
    /// Falha com [`crate::openmtr::error::OpenMTRError`] de validação se o resultado não tiver
    /// 11 dígitos ou se os verificadores não conferirem. O valor guardado é sempre nu.
    pub fn new(digits: &str) -> Result<Self, OpenMTRError> {
        let clean: String = digits.chars().filter(char::is_ascii_digit).collect();
        if Self::validar_digitos(&clean) {
            Ok(Self(clean))
        } else {
            Err(OpenMTRError::validation(format!("CPF inválido: {digits}")))
        }
    }

    /// Normaliza um CPF sem validar os dígitos verificadores.
    ///
    /// Preserva apenas os dígitos ASCII informados; útil para valores opacos de integrações.
    #[must_use]
    pub fn parse_unchecked(s: &str) -> Self {
        Self(s.chars().filter(char::is_ascii_digit).collect())
    }

    /// Os 11 dígitos do CPF, sem pontuação.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Escreve o CPF no formato canônico `NNN.NNN.NNN-NN`.
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

/// Serializa o CPF como string pontuada, igual à saída de [`fmt::Display`].
impl Serialize for Cpf {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

/// Desserializa um CPF de string pontuada ou somente dígitos, validando-o.
impl<'de> Deserialize<'de> for Cpf {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}

/// Documento de uma parte do MTR: CPF de pessoa física ou CNPJ de pessoa jurídica.
///
/// O tipo é decidido pela quantidade de dígitos após normalização (11 ou 14), não por
/// máscara. Serializa como string pontuada na variante correspondente; um `CpfCnpj`
/// desserializado round-tripa para a mesma variante.
#[derive(Clone, Debug)]
pub enum CpfCnpj {
    /// Documento de pessoa física, com 11 dígitos normalizados.
    Cpf(Cpf),
    /// Documento de pessoa jurídica, com 14 dígitos normalizados.
    Cnpj(Cnpj),
}

impl CpfCnpj {
    /// Infere o tipo pelo número de dígitos ASCII e normaliza o documento.
    ///
    /// 11 dígitos produzem [`Self::Cpf`] e 14 produzem [`Self::Cnpj`], validando os
    /// verificadores; qualquer outra quantidade falha com validação.
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

    /// Os dígitos do documento, sem pontuação, na variante correta.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Cpf(c) => c.as_str(),
            Self::Cnpj(c) => c.as_str(),
        }
    }
}

/// Escreve o documento pontuado conforme a variante (CPF ou CNPJ).
impl fmt::Display for CpfCnpj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cpf(c) => write!(f, "{c}"),
            Self::Cnpj(c) => write!(f, "{c}"),
        }
    }
}

/// Serializa o documento como string pontuada na variante correspondente.
impl Serialize for CpfCnpj {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Cpf(c) => c.serialize(serializer),
            Self::Cnpj(c) => c.serialize(serializer),
        }
    }
}

/// Desserializa um documento de 11 ou 14 dígitos, inferindo a variante e validando-o.
impl<'de> Deserialize<'de> for CpfCnpj {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}
