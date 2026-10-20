use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[derive(Default)]
pub enum TipoParte {
    #[default]
    Gerador,
    Transportador,
    Destinador,
    Armazenador,
    Outro(String),
}

impl fmt::Display for TipoParte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gerador => write!(f, "Gerador"),
            Self::Transportador => write!(f, "Transportador"),
            Self::Destinador => write!(f, "Destinador"),
            Self::Armazenador => write!(f, "Armazenador"),
            Self::Outro(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TipoManifesto {
    Padrao,
    Complementar,
    Simplificado,
    Emergencial,
    Destinacao,
    Outro(i32),
}

impl TipoManifesto {
    #[must_use]
    pub fn codigo(&self) -> i32 {
        match self {
            Self::Padrao => 1,
            Self::Complementar => 2,
            Self::Simplificado => 3,
            Self::Emergencial => 4,
            Self::Destinacao => 5,
            Self::Outro(c) => *c,
        }
    }
}

impl fmt::Display for TipoManifesto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Padrao => write!(f, "Padrão"),
            Self::Complementar => write!(f, "Complementar"),
            Self::Simplificado => write!(f, "Simplificado"),
            Self::Emergencial => write!(f, "Emergencial"),
            Self::Destinacao => write!(f, "Destinação"),
            Self::Outro(c) => write!(f, "Outro ({c})"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FormatoDownload {
    Pdf,
    Xml,
}

impl fmt::Display for FormatoDownload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pdf => write!(f, "PDF"),
            Self::Xml => write!(f, "XML"),
        }
    }
}
