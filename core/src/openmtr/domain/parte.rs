//! Atores envolvidos no transporte de resíduos (gerador, transportador,
//! destinador e armazenador temporário) nos sistemas SINIR, SIGOR e MTR-PE.

use serde::{Deserialize, Serialize};

use super::newtypes::CpfCnpj;

/// Identificação de um ator de um MTR.
///
/// O documento aceita CPF (11 dígitos) ou CNPJ (14), descarta pontuação na
/// entrada e é serializado como string pontuada por [`CpfCnpj`]; os adaptadores
/// enviam a forma numérica (`as_str()`) às APIs. A unidade é o código interno do
/// estabelecimento no sistema de origem, usado no SINIR/SIGOR e no MTR-PE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parte {
    /// CPF ou CNPJ do ator, normalizado para dígitos e validado na construção.
    pub documento: CpfCnpj,
    /// Razão social ou nome completo do ator, enviado como nome do parceiro.
    pub razao_social: String,
    /// Endereço do estabelecimento; ausente é omitido na serialização direta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endereco: Option<String>,
    /// Município do estabelecimento; ausente é omitido na serialização direta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub municipio: Option<String>,
    /// Sigla da unidade federativa do estabelecimento (ex.: `PE`), quando informada.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uf: Option<String>,
    /// Código numérico da unidade/estabelecimento no sistema; `None` é
    /// convertido em `0` nos payloads do SINIR e do SIGOR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unidade: Option<u32>,
}
