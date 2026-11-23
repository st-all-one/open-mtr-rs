use crate::openmtr::error::OpenMTRError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Quantidade de resíduo no romaneio MTR, em `f64` na unidade do item.
///
/// A unidade de medida é a do registro de resíduo (`unidade`), não um newtype aqui;
/// [`Self::new`] impõe o intervalo `[0, 1_000_000_000]`. Serializa como número JSON.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Quantidade(f64);

impl Quantidade {
    /// Aceita valores de `0` a `1_000_000_000`, falhando com validação fora desse intervalo.
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

    /// Constrói uma quantidade sem aplicar os limites, para dados já confiáveis.
    #[allow(
        dead_code,
        reason = "campo/helper mantido para compatibilidade da API e testes"
    )]
    pub(crate) fn new_unchecked(valor: f64) -> Self {
        Self(valor)
    }

    /// A quantidade numérica na unidade do item de resíduo.
    #[must_use]
    pub fn get(&self) -> f64 {
        self.0
    }
}

/// Escreve a quantidade como número decimal simples, sem formatação de unidade.
impl fmt::Display for Quantidade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
