use crate::openmtr::error::OpenMTRError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Código IBAMA de resíduo: 6 ou 8 dígitos, sem pontuação.
///
/// Identifica o resíduo nas tabelas oficiais do IBAMA e é o valor de `codigo_ibama`
/// trafegado no JSON de SINIR/MTR-PE/SIGOR. [`Self::new`] remove não dígitos ASCII e
/// exige exatamente 6 ou 8 dígitos; serializa como string numérica.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CodigoIbama(String);

impl CodigoIbama {
    /// Normaliza o código para dígitos ASCII e exige 6 ou 8 dígitos.
    ///
    /// Falha com validação para qualquer outra quantidade de dígitos.
    pub fn new(digits: &str) -> Result<Self, OpenMTRError> {
        let cleaned: String = digits.chars().filter(char::is_ascii_digit).collect();
        if cleaned.len() != 6 && cleaned.len() != 8 {
            return Err(OpenMTRError::validation(format!(
                "Código IBAMA deve ter 6 ou 8 dígitos: {digits}"
            )));
        }
        Ok(Self(cleaned))
    }

    /// Guarda o código sem normalizar nem validar, para valores já canônicos.
    #[must_use]
    pub fn parse_unchecked(s: &str) -> Self {
        Self(s.to_string())
    }

    /// O código IBAMA como string de 6 ou 8 dígitos.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Escreve o código IBAMA como string de dígitos.
impl fmt::Display for CodigoIbama {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Placa de veículo do transporte MTR, em maiúsculas.
///
/// Aceita o padrão Mercosul `ABC1D23` (3 letras, 1 dígito, 1 letra, 2 dígitos) ou o
/// padrão antigo `ABC-1234` (3 letras, hífen, 4 dígitos). A normalização converte a
/// entrada para maiúsculas; serializa como string no formato canônico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacaVeiculo(String);

impl PlacaVeiculo {
    /// Normaliza para maiúsculas e valida os formatos `ABC1D23` ou `ABC-1234`.
    ///
    /// Falha com validação se o texto não casar exatamente com um dos dois formatos.
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

    /// Converte a placa para maiúsculas sem validar o formato.
    #[must_use]
    pub fn parse_unchecked(s: &str) -> Self {
        Self(s.to_uppercase())
    }

    /// A placa normalizada, em maiúsculas.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Escreve a placa normalizada, em maiúsculas.
impl fmt::Display for PlacaVeiculo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
