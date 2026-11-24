//! Emissão, consulta e download de CDF (Certificado de Destinação Final) nos
//! sistemas SINIR, SIGOR e MTR-PE.
//!
//! As datas são civis (`Date`), sem hora nem fuso; os campos serde derivados
//! serializam como `yyyy-mm-dd` ISO 8601, e os adaptadores as convertem para epoch
//! ms no SINIR ou `AAAAMMDD` no SIGOR e MTR-PE.
//! O download devolve os bytes do arquivo, não uma URL.
use jiff::civil::Date;
use serde::{Deserialize, Serialize};

use super::mtr::Periodo;

/// Pedido de emissão de um CDF a partir de manifestos já recebidos.
///
/// Cada item enviado a `gerar_cdfs` produz um certificado no sistema de origem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GerarCdf {
    /// Números dos manifestos de origem (`manNumero` no SINIR/SIGOR, `listaMtr` no MTR-PE).
    #[serde(default)]
    pub manifestos: Vec<String>,
    /// Data civil de emissão, serializada como `yyyy-mm-dd` ISO 8601; convertida
    /// para epoch ms no SINIR e para `AAAAMMDD` no SIGOR e MTR-PE.
    pub data_emissao: Date,
    /// Nome de quem emite o certificado (`nomeResponsavel` no SINIR/SIGOR,
    /// `responsavel` no MTR-PE).
    pub responsavel: String,
    /// Observação livre da emissão; ausente vira `None` e é omitida na serialização direta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observacoes: Option<String>,
}

/// CDF consultado no sistema de origem.
///
/// A consulta existe no MTR-PE; SINIR e SIGOR respondem `not_supported`. No
/// MTR-PE, `codigo` e `numero` replicam `cdfCodigo`, `data_emissao` é a data
/// atual do adaptador e `status` fica vazio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cdf {
    /// Código do certificado no sistema de origem, usado no download.
    pub codigo: String,
    /// Número do certificado exibido no sistema; no MTR-PE replica `codigo`.
    pub numero: String,
    /// Data civil de emissão informada pelo sistema, serializada como `yyyy-mm-dd`.
    pub data_emissao: Date,
    /// Situação do certificado conforme o sistema; vazia quando a API não informa.
    pub status: String,
}

/// CDF emitido com sucesso por `gerar_cdfs`.
///
/// `codigo` identifica o certificado no download; `data_emissao` vem da resposta
/// do SINIR (`dd/MM/yyyy`) e da data atual do adaptador no SIGOR e MTR-PE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdfEmitido {
    /// Código do certificado no sistema de origem, usado no download.
    pub codigo: String,
    /// Número do certificado no sistema; vazio quando a API não fornece um separado.
    pub numero: String,
    /// Data civil de emissão do certificado, serializada como `yyyy-mm-dd`.
    pub data_emissao: Date,
    /// Situação do certificado devolvida pelo sistema; vazia quando não informada.
    pub status: String,
}

/// Critérios de consulta de CDFs (`consultar_cdfs`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FiltroCdf {
    /// Códigos de CDF a consultar; `None` não restringe a busca. O MTR-PE ignora
    /// este campo e filtra apenas por `periodo`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigos: Option<Vec<String>>,
    /// Intervalo de emissão em [`Periodo`]: cada extremo é uma
    /// [`DataMtr`](super::newtypes::DataMtr) cuja representação é `dd-MM-yyyy`;
    /// a desserialização também aceita ISO 8601 e epoch ms na entrada.
    /// `None` faz o MTR-PE usar os 30 dias anteriores à data atual do adaptador.
    /// Use `ultimos_dias_em` com uma data injetada, não `ultimos_dias`, para determinismo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub periodo: Option<Periodo>,
}
