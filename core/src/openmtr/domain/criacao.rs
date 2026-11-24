//! Criação de manifestos de transporte de resíduos nos sistemas SINIR, SIGOR e
//! MTR-PE, incluindo o manifesto complementar do SIGOR.
//!
//! As quantidades dos itens seguem a unidade da tabela `UnidadeMedida` indicada
//! por `unidade`; os códigos de resíduo vêm do catálogo do IBAMA. Datas civis
//! são `Date`/`DateTime` com serde derivado (`yyyy-mm-dd` e ISO 8601);
//! os adaptadores as convertem para epoch ms, `AAAAMMDD` ou `yyyymmdd` conforme
//! o sistema.
use jiff::civil::DateTime;
use serde::{Deserialize, Serialize};

use super::parte::Parte;
use super::residuo::ResiduoCriacao;
use super::tipo::TipoManifesto;

/// Pedido de criação de um MTR em lote (`criar_mtrs`).
///
/// Válido no SINIR, SIGOR e MTR-PE; cada item do lote é processado de forma
/// independente. O numero nacional é atribuído pelo sistema, não pelo chamador.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriarMtr {
    /// Gerador do resíduo; no MTR-PE é identificado por CNPJ e no SINIR/SIGOR por
    /// CPF/CNPJ mais unidade.
    pub gerador: Parte,
    /// Transportador do resíduo, com o mesmo par documento/unidade dos demais atores.
    pub transportador: Parte,
    /// Destinador final do resíduo; no MTR-PE seus dados também definem o CDF.
    pub destinador: Parte,
    /// Itens de resíduo do manifesto, cada qual com código IBAMA, quantidade na
    /// unidade indicada e classificação técnica. Lista vazia é permitida pelo tipo.
    #[serde(default)]
    pub residuos: Vec<ResiduoCriacao>,
    /// Nome de quem assina a criação pelo gerador (`nomeResponsavel`).
    pub responsavel: String,
    /// Observação livre da criação; `None` é omitida na serialização direta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observacoes: Option<String>,
    /// Data e hora da expedição, sem fuso; o SINIR e o SIGOR enviam epoch ms ao
    /// sistema, e o MTR-PE envia apenas a data civil `yyyymmdd`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_expedicao: Option<DateTime>,
    /// Placa do veículo; a normalização e validação ficam a cargo de cada sistema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placa_veiculo: Option<String>,
    /// Nome do motorista transportador.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motorista: Option<String>,
    /// Parte de armazenamento temporário, quando o resíduo não segue direto ao
    /// destinador; sua presença liga `possuiArmazenamentoTemporario` no SINIR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub armazenador_temporario: Option<Parte>,
    /// Tipo do manifesto; `codigo()` mapeia as variantes para os códigos numéricos
    /// do SINIR (1 a 5). O MTR-PE e o SIGOR não enviam esse campo na criação comum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tipo_manifesto: Option<TipoManifesto>,
    /// Cargo do responsável pelo gerador; usado no MTR-PE (`manifGeradorCargoResponsavel`),
    /// com padrão `Responsavel` quando ausente.
    pub cargo: Option<String>,
    /// Código de referência do chamador, enviado como `seuCodigo` no SIGOR e, se
    /// numérico, como `seuCodigoReferencia` no MTR-PE.
    pub codigo_referencia: Option<String>,
}

/// Resultado da criação de um MTR.
///
/// O SINIR preenche `numero` com `manifestoNumeroNacional` e `codigo_gerado`
/// com `codigoGerado`; o MTR-PE preenche apenas `numero`; o SIGOR preenche ambos
/// a partir de `manifestoComplementar` no complementar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtrCriado {
    /// Número nacional atribuído ao manifesto; `None` quando a API não devolve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numero: Option<String>,
    /// Código interno gerado pelo sistema; `None` quando a API não devolve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo_gerado: Option<i64>,
    /// Indica se o sistema aceitou a criação (`restResponseValido` no SINIR/SIGOR).
    pub sucesso: bool,
    /// Mensagem de sucesso ou erro devolvida pelo sistema; vazia quando ausente.
    pub mensagem: String,
}

/// Pedido de manifesto complementar no SIGOR (`/salvarManifestoComplementar`).
///
/// Agrega manifestos já emitidos para registrar transporte posterior; não é
/// suportado por SINIR ou MTR-PE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriarMtrComplementar {
    /// Nome do motorista do complementar (`nomeMotorista`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motorista: Option<String>,
    /// Placa do veículo do complementar (`placaVeiculo`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placa_veiculo: Option<String>,
    /// Observação livre do complementar.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observacoes: Option<String>,
    /// Transportador do complementar; documento e unidade são enviados ao SIGOR.
    pub transportador: Parte,
    /// Números dos manifestos originais, enviados como uma lista de `manNumero`.
    pub manifestos: Vec<String>,
}
