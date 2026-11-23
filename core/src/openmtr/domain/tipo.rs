//! Tipos enumerados que classificam partes, manifestos e downloads no MTR.
//!
//! [`TipoManifesto::codigo`] converte as variantes para os códigos numéricos usados pelo SINIR
//! (1 a 5); `Outro` preserva valores não mapeados informados pelo sistema de origem. As variantes
//! `Outro` são preservadas na serialização, permitindo round-trip de valores desconhecidos.
use serde::{Deserialize, Serialize};
use std::fmt;

/// Papel de uma parte em um manifesto MTR (gerador, transportador etc.).
///
/// Usado em [`crate::openmtr::domain::FiltroParte`] para filtrar manifestos
/// por participante; `Outro` preserva um papel não mapeado vindo do sistema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[derive(Default)]
pub enum TipoParte {
    /// Gerador do resíduo; variante padrão.
    #[default]
    Gerador,
    /// Transportador do resíduo.
    Transportador,
    /// Destinador final do resíduo.
    Destinador,
    /// Armazenador temporário do resíduo.
    Armazenador,
    /// Papel não mapeado, preservado como texto para round-trip.
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

/// Tipo de manifesto MTR, com o código numérico usado pelo SINIR.
///
/// O código é enviado como `tipoManifesto` na criação; `Outro` mantém um
/// código não mapeado para não perder informação no round-trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TipoManifesto {
    /// Manifesto padrão (código 1).
    Padrao,
    /// Manifesto complementar (código 2).
    Complementar,
    /// Manifesto simplificado (código 3).
    Simplificado,
    /// Manifesto emergencial (código 4).
    Emergencial,
    /// Manifesto de destinação (código 5).
    Destinacao,
    /// Código não mapeado, preservado para round-trip.
    Outro(i32),
}

impl TipoManifesto {
    /// Código numérico do tipo de manifesto esperado pelo SINIR.
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

/// Formato de arquivo solicitado no download de CDF.
///
/// Cada adaptador converte a variante para o valor aceito pela sua API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FormatoDownload {
    /// Arquivo em PDF.
    Pdf,
    /// Arquivo em XML.
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
