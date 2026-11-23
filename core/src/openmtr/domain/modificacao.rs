//! Alteração de MTRs já emitidos nos sistemas SINIR, SIGOR e MTR-PE.
//!
//! Os três adaptadores suportam apenas [`ModificacaoTipo::Recebimento`] por
//! enquanto; os demais tipos produzem `not_supported`/erro de validação ao serem
//! enviados. As quantidades alteradas seguem a unidade indicada no item e os
//! códigos citados vêm das tabelas do IBAMA e do sistema.

use serde::{Deserialize, Serialize};

use super::parte::Parte;
use super::residuo::ResiduoRecebido;

/// Tipo de alteração solicitada para um MTR existente.
///
/// A variante `Outro` não existe: o enum é `non_exhaustive` para permitir novos
/// tipos sem quebrar os consumidores.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ModificacaoTipo {
    /// Correção do recebimento (quantidades, classe ou tratamento dos resíduos),
    /// enviada como `/alteraRecebimento` nos três sistemas.
    Recebimento,
    /// Transferência da titularidade do manifesto; ainda sem suporte nos adaptadores.
    Titularidade,
    /// Troca ou inclusão de resíduos; ainda sem suporte nos adaptadores.
    Residuos,
}

/// Pedido de alteração de um MTR (`modificar_mtrs`).
///
/// O campo relevante depende de `tipo`: uma alteração de recebimento usa
/// `novos_residuos` e `justificativa`; mudanças de titularidade usam
/// `novo_titular`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModificarMtr {
    /// Número nacional do manifesto (`manNumero` no SINIR/SIGOR,
    /// `manifestoCodigo` convertido para inteiro no MTR-PE).
    pub numero: String,
    /// Natureza da alteração; define qual endpoint do sistema é chamado.
    pub tipo: ModificacaoTipo,
    /// Motivo da alteração, enviado como `justificativa` no SINIR/MTR-PE e como
    /// `remObservacao` no SIGOR. `None` cai em texto padrão no SINIR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justificativa: Option<String>,
    /// Novos dados dos resíduos, com código IBAMA e quantidades na unidade
    /// informada; no MTR-PE apenas o primeiro item é considerado.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub novos_residuos: Option<Vec<ResiduoRecebido>>,
    /// Nova parte titular do manifesto, identificada por CPF/CNPJ normalizado
    /// e unidade; ainda não enviada pelos adaptadores atuais.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub novo_titular: Option<Parte>,
}

/// Resultado de uma solicitação de alteração de MTR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtrModificado {
    /// Número nacional do manifesto alterado.
    pub numero: String,
    /// Indica se a solicitação foi enviada/aceita pelo sistema.
    pub modificado: bool,
    /// Mensagem devolvida pelo sistema; texto padrão quando a API não responde
    /// com um retorno próprio.
    pub mensagem: String,
}
