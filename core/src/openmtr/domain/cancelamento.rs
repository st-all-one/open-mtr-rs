//! Cancelamento de MTRs nos sistemas SINIR, SIGOR e MTR-PE.

use serde::{Deserialize, Serialize};

/// Pedido de cancelamento de um manifesto (`cancelar_mtrs`).
///
/// Enviado como `manNumero`/`justificativa` no SINIR e SIGOR (rota individual ou
/// em lote) e como `manifestoCodigo`/`justificativa` no MTR-PE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelarMtr {
    /// Número nacional do manifesto a cancelar.
    pub numero: String,
    /// Motivo do cancelamento, registrado no sistema de origem.
    pub justificativa: String,
}

/// Resultado do cancelamento de um MTR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtrCancelado {
    /// Número do manifesto cancelado; o MTR-PE o lê de `manifestoCodigo`.
    pub numero: String,
    /// Situação devolvida pelo sistema; o MTR-PE preenche `"Cancelado"` e o
    /// SINIR exige `"cancelado"` (sem diferenciar maiúsculas) para aceitar.
    pub status: String,
    /// Data do cancelamento informada como string pelo SINIR (`dataCancelamento`);
    /// `None` quando a API não fornece o valor, como no MTR-PE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_cancelamento: Option<String>,
}
