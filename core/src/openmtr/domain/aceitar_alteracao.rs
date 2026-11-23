//! Aceite ou recusa de alterações de recebimento propostas por outro parceiro
//! nos sistemas SINIR, SIGOR e MTR-PE.

use serde::{Deserialize, Serialize};

/// Decisão sobre uma alteração de recebimento de um MTR.
///
/// Enviada como `aceite` booleano no SINIR, `"a"`/`"r"` no SIGOR e `"S"`/`"N"`
/// no MTR-PE; `codigo_gerador` só é usado pelo SIGOR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AceitarAlteracao {
    /// Número nacional do manifesto alterado (`manNumero` no SINIR/SIGOR,
    /// `codigoManifesto` convertido para inteiro no MTR-PE).
    pub numero: String,
    /// `true` aceita a alteração; `false` a recusa.
    pub aceitar: bool,
    /// Código do gerador no SIGOR (`parCodigoGerador`), necessário para
    /// identificar a proposta; ausente é enviado como `0`. Ignorado por SINIR
    /// e MTR-PE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo_gerador: Option<u32>,
}

/// Resultado do aceite ou da recusa de uma alteração.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlteracaoAceita {
    /// Número do manifesto cuja alteração foi decidida.
    pub numero: String,
    /// Indica que a decisão foi registrada com aceite (`false` para recusa).
    pub aceita: bool,
    /// Mensagem devolvida pelo sistema; no SIGOR e no MTR-PE vem de `mensagem`
    /// ou `retorno`, respectivamente.
    pub mensagem: String,
}
