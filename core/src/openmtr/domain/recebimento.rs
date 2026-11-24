//! Registro do recebimento de resíduos de um MTR nos sistemas SINIR, SIGOR e
//! MTR-PE.
//!
//! A data e hora do recebimento é uma data civil sem fuso: o SINIR e o SIGOR a
//! enviam como epoch ms (`TimestampMs`), enquanto o MTR-PE envia só a data
//! civil `yyyymmdd`. As quantidades recebidas usam a unidade da tabela
//! `UnidadeMedida` indicada em cada item.

use jiff::civil::DateTime;
use serde::{Deserialize, Serialize};

use super::residuo::ResiduoRecebido;

/// Pedido de recebimento de um MTR (`receber_mtrs`).
///
/// Cada lote identifica o manifesto pelo número nacional e declara o que chegou
/// ao destinador; o aceite da alteração do recebimento é feito à parte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceberMtr {
    /// Número nacional do manifesto (`manNumero` no SINIR/SIGOR, `manifestoCodigo` no MTR-PE).
    pub numero: String,
    /// Data e hora civil do recebimento, sem fuso; serializada por serde como
    /// ISO 8601 e convertida a epoch ms no SINIR/SIGOR ou `yyyymmdd` no MTR-PE.
    pub data_recebimento: DateTime,
    /// Nome de quem recebe os resíduos pelo destinador (`nomeResponsavel` no
    /// SINIR, `nomeResponsavelRecebimento` no SIGOR, `recebimentoMtrResponsavel`
    /// no MTR-PE).
    pub responsavel: String,
    /// Observação livre do recebimento; `None` é omitida na serialização direta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observacoes: Option<String>,
    /// Nome do motorista no SIGOR; o SINIR e o MTR-PE não enviam este campo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motorista: Option<String>,
    /// Placa do veículo no SIGOR; o SINIR e o MTR-PE não enviam este campo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placa_veiculo: Option<String>,
    /// Itens efetivamente recebidos, com código IBAMA, quantidade recebida na
    /// unidade informada e eventuais campos de alteração. Lista vazia é
    /// convertida em ausência de lista no payload do SINIR.
    #[serde(default)]
    pub residuos_recebidos: Vec<ResiduoRecebido>,
}

/// Resultado do recebimento de um MTR.
///
/// O SINIR e o SIGOR devolvem o número e o indicador de sucesso na resposta; o
/// MTR-PE replica o número da requisição e usa `retornoCodigo == 0` como sucesso.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtrRecebido {
    /// Número nacional do manifesto recebido.
    pub numero: String,
    /// Indica se o recebimento foi aceito pelo sistema.
    pub recebido: bool,
    /// Mensagem da API; `None` quando o sistema não fornece uma (`retorno` vazio
    /// no MTR-PE).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mensagem: Option<String>,
}
