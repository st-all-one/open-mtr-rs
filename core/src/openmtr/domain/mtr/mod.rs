//! Tipos de domínio de manifestos de transporte de resíduos (MTR) compartilhados entre bindings.
//!
//! [`Mtr`] é a representação canônica de um manifesto do SINIR, MTR-PE ou SIGOR: os adaptadores
//! convertem a situação de cada sistema em [`StatusMtr`] e as datas recebidas como epoch ms ou
//! em formato local em datas civis. Consultas usam [`FiltroMtr`] e [`Periodo`], e cada manifesto
//! trafega como item de `dados` em [`crate::openmtr::resposta::Resposta`].
use jiff::civil::Date;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::newtypes::CpfCnpj;
use super::parte::Parte;
use super::residuo::Residuo;
use super::tipo::TipoParte;

mod periodo;
pub use periodo::Periodo;

/// Manifesto de transporte de resíduos normalizado entre SINIR, MTR-PE e SIGOR.
///
/// Datas são civis, sem fuso horário; campos opcionais ausentes na origem ficam `None` e são
/// omitidos do JSON. Em consultas, cada `Mtr` costuma vir dentro de um envelope
/// [`crate::openmtr::resposta::Resposta`].
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Mtr {
    /// Número nacional do manifesto (`manNumero` no SINIR/SIGOR, `manifestoCodigo` no MTR-PE).
    ///
    /// Identifica o MTR na consulta por números; fica vazio quando a origem não informa.
    pub numero: String,
    /// Código de barras completo do manifesto, quando o sistema o fornece
    /// (ex.: MTR-PE). Fica vazio em sistemas que não expõem esse código
    /// (ex.: SINIR, SIGOR).
    #[serde(default)]
    pub barcode: String,
    /// Situação canônica do manifesto, convertida da origem em [`StatusMtr`] e serializada como string.
    pub status: StatusMtr,
    /// Gerador dos resíduos, quando a resposta da origem o fornece.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gerador: Option<Parte>,
    /// Transportador dos resíduos, quando a resposta da origem o fornece.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transportador: Option<Parte>,
    /// Destinador final dos resíduos, quando a resposta da origem o fornece.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destinador: Option<Parte>,
    /// Itens de resíduo do manifesto; cada `quantidade` está na unidade de romaneio do item.
    ///
    /// Vetor vazio quando a listagem da origem não detalha os resíduos.
    #[serde(default)]
    pub residuos: Vec<Residuo>,
    /// Data civil de criação do manifesto, serializada como string ISO 8601 (`YYYY-MM-DD`).
    ///
    /// A origem pode fornecer epoch ms (SINIR/SIGOR) ou data local `YYYYMMDD` (MTR-PE).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_criacao: Option<Date>,
    /// Data civil do recebimento pelo destinador, serializada como ISO 8601 (`YYYY-MM-DD`).
    ///
    /// Ausente enquanto o manifesto ainda não foi recebido.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_recebimento: Option<Date>,

    // --- Campos adicionais (GAP 3) ---
    /// Observação livre do manifesto informada pela origem.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observacao: Option<String>,
    /// Nome do motorista responsável pelo transporte.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motorista: Option<String>,
    /// Placa do veículo usado no transporte, como texto informado pela origem.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placa: Option<String>,
    /// Código numérico do tipo de manifesto na origem, conforme
    /// [`crate::openmtr::domain::tipo::TipoManifesto::codigo`] (1 = padrão, 2 = complementar, ...).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tipo_manifesto: Option<i32>,
    /// Sigla da UF do manifesto informada pela origem (ex.: `PE`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estado_uf: Option<String>,
    /// Data civil de expedição do manifesto, serializada como ISO 8601 (`YYYY-MM-DD`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_expedicao: Option<Date>,
    /// Indica se o manifesto possui armazenamento temporário; `None` se a origem não informa.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub armazenamento_temporario: Option<bool>,
    /// Hash do manifesto na origem (`manHashCode` no SINIR/SIGOR), usado na conferência do documento.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Indica se existe solicitação de alteração de recebimento pendente na origem.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tem_alteracao_pendente: Option<bool>,
    /// Número estadual do manifesto, distinto do número nacional em [`Self::numero`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numero_estadual: Option<String>,
    /// Código numérico estadual do manifesto na origem (`manCodigoEstadual`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo_estadual: Option<i64>,
}

/// Situação canônica de um [`Mtr`], unificando os estados informados por SINIR, MTR-PE e SIGOR.
///
/// A serialização é sempre uma string via `Display`: `EmTransporte` vira `"Em Transporte"` e
/// [`Self::Outro`] vira o próprio texto recebido, sem invólucro — é assim que status não mapeados
/// sobrevivem ao envio para JSON. A leitura usa o `Deserialize` derivado, que espera os nomes das
/// variantes (ex.: `"EmTransporte"`) ou `{"Outro":"..."}` — não há simetria garantida de round-trip
/// para status livres; o enum é `non_exhaustive`, então trate `Outro` como alternativa.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
#[derive(Default)]
pub enum StatusMtr {
    /// Manifesto ativo na origem; variante padrão quando o status não é informado.
    #[default]
    Ativo,
    /// Manifesto em transporte; serializado como `"Em Transporte"`.
    EmTransporte,
    /// Manifesto recebido pelo destinador.
    Recebido,
    /// Manifesto cancelado na origem.
    Cancelado,
    /// Manifesto com prazo expirado.
    Expirado,
    /// Status livre ainda não mapeado, com a descrição recebida da origem (ex.: `Código 9`).
    ///
    /// Serializado como o próprio texto, preservando status desconhecidos do SINIR/SIGOR/MTR-PE.
    Outro(String),
}

impl Serialize for StatusMtr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl fmt::Display for StatusMtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ativo => write!(f, "Ativo"),
            Self::EmTransporte => write!(f, "Em Transporte"),
            Self::Recebido => write!(f, "Recebido"),
            Self::Cancelado => write!(f, "Cancelado"),
            Self::Expirado => write!(f, "Expirado"),
            Self::Outro(s) => write!(f, "{s}"),
        }
    }
}

/// Critérios de consulta de manifestos, enviados à API conforme o sistema de origem.
///
/// [`Self::periodo`] aceita o atalho `{"dias": N}`, resolvido na desserialização pelo relógio do
/// sistema; para determinismo construa [`Periodo`] com [`Periodo::ultimos_dias_em`] ou datas
/// explícitas. `pagina` é 1-based e `tamanho` controla a paginação do SINIR, convertida em offset.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FiltroMtr {
    /// Números exatos de manifestos a consultar.
    ///
    /// Uma lista não vazia seleciona a busca pelo detalhe de cada manifesto em vez da listagem
    /// por período.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numeros: Option<Vec<String>>,
    /// Intervalo da consulta; `{"dias": N}` torna-se `Periodo::ultimos_dias` ao ser lido.
    ///
    /// A resolução usa o relógio do sistema no instante da desserialização: use
    /// [`Periodo::ultimos_dias_em`] quando a data de referência precisar ser controlada.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub periodo: Option<Periodo>,
    /// Papel do parceiro na consulta; no SINIR determina o código `tipoParceiro` da requisição.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parte: Option<FiltroParte>,
    /// Número da página, começando em 1; no SINIR é convertido no offset `(pagina - 1) * tamanho`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagina: Option<u32>,
    /// Quantidade de registros por página; sem ele o SINIR não envia parâmetros de paginação.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tamanho: Option<u32>,
    /// Enriquece cada MTR com dados detalhados (resíduos, CNPJ, campos completos) via requisições
    /// paralelas a `GET /retornaManifesto/{numero}` no SINIR.
    ///
    /// Cada manifesto detalhado custa uma requisição adicional à consulta da listagem.
    #[serde(default)]
    pub detalhado: bool,
}
/// Restrição por parceiro aplicada à consulta de manifestos.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FiltroParte {
    /// Papel do parceiro no manifesto; no SINIR é convertido no código `tipoParceiro`.
    pub tipo: TipoParte,
    /// CPF ou CNPJ do parceiro, normalizado para dígitos; ausente quando a consulta não restringe documento.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documento: Option<CpfCnpj>,
    /// Código numérico da unidade do parceiro na origem (`uniCodigo`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unidade: Option<u32>,
}

#[cfg(test)]
mod tests;
