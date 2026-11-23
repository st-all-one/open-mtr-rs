use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::openmtr::error::OpenMTRError;

/// Envelope padrão devolvido por todas as operações do cliente openMTR.
///
/// `id` identifica a operação e é fornecido pelo gerador injetado no cliente
/// (via [`OpenMtrClientBuilder::id_generator`](crate::openmtr::open_mtr_client::OpenMtrClientBuilder::id_generator)),
/// o que torna o envelope reproduzível em testes. `ok`/`status` descrevem o
/// resultado, `dados` traz os registros e `erros` os problemas por item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resposta<T> {
    /// Identificador da operação (UUID), oriundo do gerador de IDs do cliente.
    pub id: Uuid,
    /// Indica se a operação foi concluída com sucesso.
    pub ok: bool,
    /// Código HTTP lógico do resultado (por exemplo 200 ou 400).
    pub status: u16,
    /// Mensagem de status legível; pode ser vazia quando o resultado é expresso só por `ok`.
    pub mensagem: String,
    /// Registros devolvidos pela operação; vazio quando não há dados.
    pub dados: Vec<T>,
    /// Problemas encontrados, cada um com detalhes RFC 9457 (problem+json).
    pub erros: Vec<OpenMTRError>,
    /// Total de registros disponíveis (para paginação).
    /// Preenchido quando a API retorna totalRecords.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}

impl<T> Resposta<T> {
    /// Cria um envelope de sucesso (status 200, sem erros e sem mensagem).
    #[must_use]
    pub fn sucesso(id: Uuid, dados: Vec<T>) -> Self {
        Self {
            id,
            ok: true,
            status: 200,
            mensagem: String::new(),
            dados,
            erros: Vec::new(),
            total: None,
        }
    }

    /// Cria um envelope de falha (status 400) contendo os erros informados.
    #[must_use]
    pub fn com_erros(id: Uuid, erros: Vec<OpenMTRError>) -> Self {
        Self {
            id,
            ok: false,
            status: 400,
            mensagem: String::new(),
            dados: Vec::new(),
            erros,
            total: None,
        }
    }

    /// Cria um envelope de sucesso com uma mensagem descritiva.
    pub fn com_mensagem(id: Uuid, dados: Vec<T>, mensagem: impl Into<String>) -> Self {
        Self {
            id,
            ok: true,
            status: 200,
            mensagem: mensagem.into(),
            dados,
            erros: Vec::new(),
            total: None,
        }
    }

    /// Cria um envelope de sucesso com o total de registros disponíveis (paginação).
    #[must_use]
    pub fn com_total(id: Uuid, dados: Vec<T>, total: i64) -> Self {
        Self {
            id,
            ok: true,
            status: 200,
            mensagem: String::new(),
            dados,
            erros: Vec::new(),
            total: Some(total),
        }
    }

    /// Retorna `true` quando há problemas registrados em `erros`.
    #[must_use]
    pub fn tem_erros(&self) -> bool {
        !self.erros.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resposta_sucesso() {
        let id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
        let r = Resposta::sucesso(id, vec![1, 2, 3]);
        assert!(r.ok);
        assert_eq!(r.status, 200);
        assert_eq!(r.dados, vec![1, 2, 3]);
        assert!(r.erros.is_empty());
    }

    #[test]
    fn test_resposta_com_erros() {
        let id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
        let e = OpenMTRError::validation("campo obrigatório");
        let r: Resposta<i32> = Resposta::com_erros(id, vec![e]);
        assert!(!r.ok);
        assert!(r.tem_erros());
        assert!(r.dados.is_empty());
    }

    #[test]
    fn test_resposta_com_mensagem() {
        let id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
        let r = Resposta::sucesso(id, vec!["a"]);
        assert_eq!(r.mensagem, "");
        let r = Resposta::com_mensagem(id, vec!["a"], "criado com sucesso");
        assert_eq!(r.mensagem, "criado com sucesso");
    }
}
