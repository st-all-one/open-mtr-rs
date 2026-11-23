//! Tabelas de domínio do MTR: listas de apoio de SINIR, MTR-PE e SIGOR.
//!
//! Os códigos `u32` destas tabelas são os mesmos enviados nos campos de
//! classificação de [`crate::openmtr::domain::Residuo`] e de seus tipos de
//! criação/recebimento. As tabelas estáticas usam `BTreeMap`: iteração e
//! serialização seguem a ordem crescente das chaves, de forma determinística.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::LazyLock;

/// Forma de acondicionamento do resíduo (tabela `tiaCodigo`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Acondicionamento {
    /// Código do acondicionamento na tabela do sistema de origem.
    pub codigo: u32,
    /// Descrição do acondicionamento (ex.: "Container", "Tambor").
    pub descricao: String,
}

/// Unidade de medida da quantidade de resíduo (tabela `uniCodigo`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnidadeMedida {
    /// Código da unidade na tabela do sistema de origem.
    pub codigo: u32,
    /// Descrição da unidade (ex.: "Quilograma").
    pub descricao: String,
    /// Sigla da unidade (ex.: "KG"), usada na apresentação ao usuário.
    pub sigla: String,
}

/// Classe de resíduo conforme a norma ambiental aplicável (tabela `claCodigo`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClasseResiduo {
    /// Código da classe na tabela do sistema de origem.
    pub codigo: u32,
    /// Descrição da classe (ex.: "Resíduo classe I").
    pub descricao: String,
    /// Resolução normativa que define a classe, quando o sistema a informa.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolucao: Option<String>,
}

/// Estado físico do resíduo (tabela `tieCodigo`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstadoFisico {
    /// Código do estado na tabela do sistema de origem.
    pub codigo: u32,
    /// Descrição do estado (ex.: "Sólido", "Líquido").
    pub descricao: String,
}

/// Tratamento dado ao resíduo (tabela `traCodigo`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TratamentoResiduo {
    /// Código do tratamento na tabela do sistema de origem.
    pub codigo: u32,
    /// Descrição do tratamento (ex.: "Reciclagem", "Incineracao").
    pub descricao: String,
}

/// Compatibilidade IBAMA entre códigos de acondicionamento e estados físicos.
///
/// A chave é o código do estado físico (ver [`ESTADOS_FISICOS`]); o valor é a
/// lista de códigos de acondicionamento [`Acondicionamento`] permitidos para
/// aquele estado. Por exemplo, o estado 4 (Gasoso) aceita apenas o código 9.
/// É um fallback estático: `BTreeMap` mantém chaves em ordem crescente, tornando
/// iteração e serialização determinísticas; os valores da API prevalecem.
pub static COMPATIBILIDADE_ACOND: LazyLock<BTreeMap<u32, Vec<u32>>> = LazyLock::new(|| {
    let mut m = BTreeMap::new();
    m.insert(1, vec![1, 2, 3, 6, 8]);
    m.insert(2, vec![2, 5, 9]);
    m.insert(3, vec![2, 5, 6]);
    m.insert(4, vec![9]);
    m.insert(5, vec![1, 2, 5, 9]);
    m
});

/// Nomes dos estados físicos indexados pelo código IBAMA do estado.
///
/// Fallback offline para a lista de [`EstadoFisico`]. O `BTreeMap` garante
/// iteração e serialização em ordem crescente de código (1 a 5: Sólido,
/// Líquido, Pastoso, Gasoso, Multifásico); os valores retornados pela API
/// do sistema de origem prevalecem sobre estes.
pub static ESTADOS_FISICOS: LazyLock<BTreeMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = BTreeMap::new();
    m.insert(1, "Sólido");
    m.insert(2, "Líquido");
    m.insert(3, "Pastoso");
    m.insert(4, "Gasoso");
    m.insert(5, "Multifásico");
    m
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compatibilidade_completa() {
        for cod in 1..=5 {
            let comp = COMPATIBILIDADE_ACOND.get(&cod);
            assert!(comp.is_some(), "Estado {cod} sem compatibilidade");
            assert!(!comp.unwrap().is_empty(), "Estado {cod} com lista vazia");
        }
    }

    #[test]
    fn test_gasoso_so_tanque() {
        let comp = COMPATIBILIDADE_ACOND.get(&4).unwrap();
        assert_eq!(comp.as_slice(), &[9]);
    }

    #[test]
    fn test_solido_tem_container() {
        let comp = COMPATIBILIDADE_ACOND.get(&1).unwrap();
        assert!(comp.contains(&1));
    }

    #[test]
    fn test_estados_fisicos_completos() {
        assert_eq!(ESTADOS_FISICOS.len(), 5);
        assert_eq!(*ESTADOS_FISICOS.get(&1).unwrap(), "Sólido");
        assert_eq!(*ESTADOS_FISICOS.get(&5).unwrap(), "Multifásico");
    }
}
