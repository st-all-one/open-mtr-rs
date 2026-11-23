//! Itens de resíduo transportado em um manifesto MTR.
//!
//! Os códigos IBAMA identificam o resíduo no catálogo oficial; quantidades são expressas na
//! `unidade` informada no item (tabela [`crate::openmtr::domain::tabela::UnidadeMedida`]). As
//! variantes de criação, recebimento e leitura refletem os campos exigidos por cada operação nos
//! sistemas SINIR, MTR-PE e SIGOR.
use serde::{Deserialize, Serialize};

/// Item de resíduo de um manifesto lido de SINIR, MTR-PE ou SIGOR.
///
/// `quantidade` está na unidade da tabela `UnidadeMedida` indicada por
/// `unidade`; `codigo_ibama` é o código do resíduo no catálogo do IBAMA.
/// Campos ausentes na resposta do sistema ficam como `None` e são omitidos
/// na serialização.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Residuo {
    /// Código IBAMA do resíduo no catálogo oficial.
    ///
    /// É a chave usada para consultar [`ResiduoIbama`].
    pub codigo_ibama: String,
    /// Descrição do resíduo conforme o sistema de origem, quando fornecida.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descricao: Option<String>,
    /// Quantidade expedida do resíduo, na unidade indicada por `unidade`.
    pub quantidade: f64,
    /// Código da unidade de medida na tabela `UnidadeMedida` (SINIR/SIGOR/MTR-PE).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unidade: Option<u32>,
    /// Código do estado físico na tabela `EstadoFisico` (SINIR/SIGOR/MTR-PE).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estado_fisico: Option<u32>,
    /// Código da classe do resíduo na tabela `ClasseResiduo`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classe: Option<u32>,
    /// Código do acondicionamento na tabela `Acondicionamento` (SINIR/SIGOR/MTR-PE).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acondicionamento: Option<u32>,
    /// Código do tratamento na tabela `TratamentoResiduo`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tratamento: Option<u32>,
    /// Quantidade efetivamente recebida pelo destinador, na unidade de `unidade`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantidade_recebida: Option<f64>,
    /// Densidade informada no manifesto, quando o sistema a expõe.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub densidade: Option<f64>,
    /// Número ONU (transporte de produtos perigosos), quando informado.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numero_onu: Option<String>,
    /// Classe de risco do resíduo para transporte, quando informada.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classe_risco: Option<String>,
    /// Nome de embarque do resíduo, quando informado.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nome_embarque: Option<String>,
    /// Código do grupo de embalagem na tabela do sistema de origem.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grupo_embalagem: Option<u32>,
    /// Código interno do item no sistema do gerador.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo_interno: Option<String>,
    /// Descrição interna do item no sistema do gerador.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descricao_interna: Option<String>,
    /// Observação livre do item de resíduo (ex.: `CX 574`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observacao: Option<String>,
}

/// Item de resíduo enviado ao criar um MTR no SINIR, MTR-PE ou SIGOR.
///
/// Os códigos de classificação são obrigatórios e devem vir das respectivas
/// tabelas de domínio; `quantidade` é expressa na unidade de `unidade`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResiduoCriacao {
    /// Código IBAMA do resíduo no catálogo oficial.
    pub codigo_ibama: String,
    /// Quantidade a transportar, na unidade indicada por `unidade`.
    pub quantidade: f64,
    /// Código da unidade de medida na tabela `UnidadeMedida`.
    pub unidade: u32,
    /// Código do estado físico na tabela `EstadoFisico`.
    pub estado_fisico: u32,
    /// Código da classe do resíduo na tabela `ClasseResiduo`.
    pub classe: u32,
    /// Código do acondicionamento na tabela `Acondicionamento`.
    pub acondicionamento: u32,
    /// Código do tratamento na tabela `TratamentoResiduo`.
    pub tratamento: u32,
    /// Descrição interna do item no sistema do gerador.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descricao_interna: Option<String>,
    /// Código interno do item no sistema do gerador.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo_interno: Option<String>,
    /// Densidade informada no manifesto, quando disponível.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub densidade: Option<f64>,
    /// Número ONU para transporte de produtos perigosos, quando aplicável.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numero_onu: Option<String>,
    /// Classe de risco do resíduo para transporte, quando aplicável.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classe_risco: Option<String>,
    /// Nome de embarque do resíduo, quando informado.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nome_embarque: Option<String>,
    /// Código do grupo de embalagem na tabela do sistema de origem.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grupo_embalagem: Option<u32>,
}

/// Item de resíduo informado no recebimento de um MTR.
///
/// Os campos `*_novo` registram a reclassificação feita pelo destinador
/// (novo código IBAMA, tratamento ou classe); ausência significa que o
/// resíduo foi recebido conforme classificado na origem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResiduoRecebido {
    /// Código IBAMA do resíduo conforme classificado na origem.
    pub codigo_ibama: String,
    /// Quantidade efetivamente recebida, na unidade indicada por `unidade`.
    pub quantidade_recebida: f64,
    /// Código da unidade de medida na tabela `UnidadeMedida`.
    pub unidade: u32,
    /// Código do estado físico na tabela `EstadoFisico`, quando informado.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estado_fisico: Option<u32>,
    /// Código da classe do resíduo na tabela `ClasseResiduo`, quando informado.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classe: Option<u32>,
    /// Código do acondicionamento na tabela `Acondicionamento`, quando informado.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acondicionamento: Option<u32>,
    /// Código do tratamento na tabela `TratamentoResiduo`, quando informado.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tratamento: Option<u32>,
    /// Novo código IBAMA atribuído pelo destinador no recebimento.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo_ibama_novo: Option<String>,
    /// Novo código de tratamento atribuído pelo destinador.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tratamento_novo: Option<u32>,
    /// Nova classe de resíduo atribuída pelo destinador.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classe_nova: Option<u32>,
    /// Densidade informada no recebimento, quando disponível.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub densidade: Option<f64>,
}

/// Registro do catálogo de resíduos do IBAMA.
///
/// O código é a chave oficial do resíduo; a descrição acompanha a lista
/// retornada por SINIR, MTR-PE ou SIGOR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResiduoIbama {
    /// Código oficial do resíduo no catálogo do IBAMA.
    pub codigo_ibama: String,
    /// Descrição oficial do resíduo no catálogo do IBAMA.
    pub descricao: String,
}

impl ResiduoIbama {
    /// Indica resíduo perigoso pelo prefixo do código IBAMA.
    ///
    /// Após remover espaços e converter para maiúsculas, códigos iniciados
    /// por `F`, `K`, `P` ou `U` são considerados perigosos.
    #[must_use]
    pub fn is_perigoso(&self) -> bool {
        let code = self.codigo_ibama.trim().to_uppercase();
        code.starts_with('F')
            || code.starts_with('K')
            || code.starts_with('P')
            || code.starts_with('U')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_residuo_perigoso() {
        let r = ResiduoIbama {
            codigo_ibama: "F001".to_string(),
            descricao: "Resíduo perigoso".to_string(),
        };
        assert!(r.is_perigoso());
    }

    #[test]
    fn test_residuo_nao_perigoso() {
        let r = ResiduoIbama {
            codigo_ibama: "A001".to_string(),
            descricao: "Resíduo não perigoso".to_string(),
        };
        assert!(!r.is_perigoso());
    }

    #[test]
    fn test_residuo_criacao_serialize() {
        let r = ResiduoCriacao {
            codigo_ibama: "F001".to_string(),
            quantidade: 100.0,
            unidade: 1,
            estado_fisico: 1,
            classe: 1,
            acondicionamento: 1,
            tratamento: 1,
            descricao_interna: None,
            codigo_interno: None,
            densidade: None,
            numero_onu: None,
            classe_risco: None,
            nome_embarque: None,
            grupo_embalagem: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"codigo_ibama\""));
        assert!(json.contains("\"quantidade\""));
    }
}
