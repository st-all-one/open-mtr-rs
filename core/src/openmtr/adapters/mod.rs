use std::sync::Arc;

use serde_json::Value;

use crate::openmtr::client::http::{MtrHttpClient, RawResponse};
use crate::openmtr::config::Config;
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::time_util::{Clock, IdGenerator, SystemClock, UuidV7Generator};

mod trait_def;

pub use trait_def::MtrAdapter;

/// Resultado de um item dentro de uma operação em lote.
///
/// Um item é ou um valor de domínio (`Ok`) ou um problema RFC 9457 com o
/// índice que falhou (`Erro`), permitindo sucesso parcial sem perder os
/// registros bons.
#[derive(Debug, Clone)]
#[allow(
    clippy::large_enum_variant,
    reason = "Ok/Erro são o contrato de resultado em lote; boxing esconderia o domínio"
)]
pub enum ItemResult<T> {
    /// Item processado com sucesso, com seu valor de domínio.
    Ok(T),
    /// Item que falhou, com o índice na entrada e o problema encontrado.
    Erro {
        /// Posição do item na entrada (base zero).
        indice: usize,
        /// Problema RFC 9457 (problem+json) ocorrido no item.
        problema: OpenMTRError,
    },
}

/// Resultado agregado de uma operação em lote.
#[derive(Debug, Clone)]
pub struct LoteResult<T> {
    /// Resultado individual de cada item da entrada.
    pub itens: Vec<ItemResult<T>>,
    /// Total de registros disponíveis (paginação)
    pub total: Option<i64>,
}

impl<T> LoteResult<T> {
    /// Cria um lote a partir dos resultados dos itens, sem total de paginação.
    #[must_use]
    pub fn new(itens: Vec<ItemResult<T>>) -> Self {
        Self { itens, total: None }
    }

    /// Cria um lote com o total de registros disponíveis (paginação).
    #[must_use]
    pub fn com_total(itens: Vec<ItemResult<T>>, total: i64) -> Self {
        Self {
            itens,
            total: Some(total),
        }
    }

    /// Retorna referências aos valores dos itens que sucederam.
    #[must_use]
    pub fn sucessos(&self) -> Vec<&T> {
        self.itens
            .iter()
            .filter_map(|i| match i {
                ItemResult::Ok(val) => Some(val),
                ItemResult::Erro { .. } => None,
            })
            .collect()
    }

    /// Retorna referências aos problemas dos itens que falharam.
    #[must_use]
    pub fn erros(&self) -> Vec<&OpenMTRError> {
        self.itens
            .iter()
            .filter_map(|i| match i {
                ItemResult::Ok(_) => None,
                ItemResult::Erro { problema, .. } => Some(problema),
            })
            .collect()
    }

    /// Conta quantos itens sucederam.
    #[must_use]
    pub fn total_ok(&self) -> usize {
        self.itens
            .iter()
            .filter(|item| matches!(item, ItemResult::Ok(_)))
            .count()
    }

    /// Conta quantos itens falharam.
    #[must_use]
    pub fn total_erros(&self) -> usize {
        self.itens
            .iter()
            .filter(|item| matches!(item, ItemResult::Erro { .. }))
            .count()
    }

    /// Cria um lote todo bem-sucedido a partir de valores de domínio.
    pub fn from_vec(dados: Vec<T>) -> Self {
        let itens = dados.into_iter().map(ItemResult::Ok).collect();
        Self { itens, total: None }
    }

    /// Cria um lote todo bem-sucedido com o total de registros (paginação).
    pub fn from_vec_com_total(dados: Vec<T>, total: i64) -> Self {
        let itens = dados.into_iter().map(ItemResult::Ok).collect();
        Self {
            itens,
            total: Some(total),
        }
    }
}

/// Cliente HTTP concreto usado pelos adaptadores (`MtrHttpClient`).
pub type HttpClient = MtrHttpClient;

/// Contexto compartilhado por todas as operações dos adaptadores.
///
/// Reúne o cliente HTTP, a configuração do sistema e os pontos de injeção de
/// tempo e identificadores. Os métodos [`Self::today`] e [`Self::new_id`] são a
/// única fonte de data atual e de `id` do envelope [`Resposta`](crate::openmtr::resposta::Resposta):
/// com [`Self::with_determinism`], ambos vêm dos doubles injetados.
pub struct MtrCtx {
    /// Cliente HTTP usado para as chamadas à API do sistema.
    pub http: HttpClient,
    /// Configuração resolvida (sistema, URL base, token, timeout, RPS).
    pub config: Config,
    /// Relógio injetado, fonte de `now()` e `today()`.
    pub clock: Arc<dyn Clock>,
    /// Gerador injetado, fonte dos UUIDs de envelope e de erro.
    pub id_generator: Arc<dyn IdGenerator>,
}

impl MtrCtx {
    /// Cria um contexto com relógio e gerador de IDs padrão.
    #[must_use]
    pub fn new(http: HttpClient, config: Config) -> Self {
        Self {
            http,
            config,
            clock: Arc::new(SystemClock),
            id_generator: Arc::new(UuidV7Generator),
        }
    }

    /// Cria um contexto com relógio e gerador de IDs injetados.
    #[must_use]
    pub fn with_determinism(
        http: HttpClient,
        config: Config,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            http,
            config,
            clock,
            id_generator,
        }
    }

    /// Instante atual segundo o relógio injetado.
    #[must_use]
    pub fn now(&self) -> chrono::DateTime<chrono::Utc> {
        self.clock.now()
    }

    /// Data civil (sem fuso) segundo o relógio injetado.
    #[must_use]
    pub fn today(&self) -> chrono::NaiveDate {
        self.clock.today()
    }

    /// Novo identificador (UUID) segundo o gerador injetado.
    #[must_use]
    pub fn new_id(&self) -> uuid::Uuid {
        self.id_generator.new_id()
    }
}

/// Valida o arquivo baixado antes de devolvê-lo: não vazio, tamanho mínimo e assinatura do formato.
///
/// `expected` é o formato esperado (por exemplo `PDF`); retorna
/// `Some(OpenMTRError)` com os detalhes da validação que falhou, ou `None`.
pub(crate) fn validate_download(response: &RawResponse, expected: &str) -> Option<OpenMTRError> {
    if response.bytes.is_empty() {
        return Some(OpenMTRError::validation("Download retornou arquivo vazio"));
    }

    if response.bytes.len() < 100 {
        return Some(OpenMTRError::validation(format!(
            "Arquivo muito pequeno ({} bytes) para ser um {expected} válido",
            response.bytes.len()
        )));
    }

    if let Err(msg) = response.validate(expected) {
        return Some(OpenMTRError::validation(msg));
    }

    None
}

/// Faz um `GET` e devolve o corpo como `serde_json::Value`.
///
/// `label` identifica a operação em logs e mensagens de erro.
#[allow(
    dead_code,
    reason = "campo/helper mantido para compatibilidade da API e testes"
)]
pub(crate) async fn get_value(
    ctx: &MtrCtx,
    path: &str,
    label: &str,
) -> Result<Value, OpenMTRError> {
    ctx.http.get::<Value>(path, label).await
}

/// Faz um `POST` de `body` e devolve o corpo como `serde_json::Value`.
///
/// `label` identifica a operação em logs e mensagens de erro.
#[allow(
    dead_code,
    reason = "campo/helper mantido para compatibilidade da API e testes"
)]
pub(crate) async fn post_value(
    ctx: &MtrCtx,
    path: &str,
    body: &(impl serde::Serialize + Sync),
    label: &str,
) -> Result<Value, OpenMTRError> {
    ctx.http.post::<Value, _>(path, body, label).await
}

pub(crate) mod mtr_pe;
pub(crate) mod sigor;
pub(crate) mod sinir;
