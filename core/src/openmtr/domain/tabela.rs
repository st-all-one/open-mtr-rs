use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Acondicionamento {
    pub codigo: u32,
    pub descricao: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnidadeMedida {
    pub codigo: u32,
    pub descricao: String,
    pub sigla: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClasseResiduo {
    pub codigo: u32,
    pub descricao: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolucao: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstadoFisico {
    pub codigo: u32,
    pub descricao: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TratamentoResiduo {
    pub codigo: u32,
    pub descricao: String,
}

pub static COMPATIBILIDADE_ACOND: LazyLock<BTreeMap<u32, Vec<u32>>> = LazyLock::new(|| {
    let mut m = BTreeMap::new();
    m.insert(1, vec![1, 2, 3, 6, 8]);
    m.insert(2, vec![2, 5, 9]);
    m.insert(3, vec![2, 5, 6]);
    m.insert(4, vec![9]);
    m.insert(5, vec![1, 2, 5, 9]);
    m
});

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
