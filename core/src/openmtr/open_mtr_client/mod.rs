//! Cliente de alto nível e seu builder, com injeção de relógio e gerador de IDs.

use std::sync::Arc;

use crate::openmtr::adapters::{MtrAdapter, MtrCtx};
use crate::openmtr::error::OpenMTRError;
use crate::openmtr::resposta::Resposta;

mod builder;
mod exec;
mod ops;

#[cfg(test)]
mod tests;

pub use builder::OpenMtrClientBuilder;

/// Resultado de uma operação do cliente: envelope [`Resposta`] ou erro RFC 9457.
pub type MtrResult<T> = Result<Resposta<T>, OpenMTRError>;

/// Cliente de alto nível para operações de manifestos de transporte de resíduos.
///
/// Enumera as operações disponíveis em todos os sistemas suportados e delega
/// a tradução de protocolo ao adaptador escolhido. Cada chamada produz um
/// [`Resposta`] cujo `id` vem do gerador determinístico do contexto.
pub struct OpenMtrClient {
    pub(crate) ctx: Arc<MtrCtx>,
    pub(crate) adapter: Box<dyn MtrAdapter>,
}

impl OpenMtrClient {
    /// Monta o cliente diretamente a partir de um contexto e um adaptador.
    #[must_use]
    pub fn from_parts(ctx: Arc<MtrCtx>, adapter: Box<dyn MtrAdapter>) -> Self {
        Self { ctx, adapter }
    }

    /// Inicia a configuração fluente do cliente.
    #[must_use]
    pub fn builder() -> OpenMtrClientBuilder {
        OpenMtrClientBuilder::default()
    }

    /// Nome legível do sistema associado ao cliente (por exemplo `SINIR Nacional`).
    #[must_use]
    pub fn system_name(&self) -> &'static str {
        self.ctx.config.system.nome()
    }

    /// Retorna o access token atual, sem o prefixo `Bearer`.
    pub async fn access_token(&self) -> Result<String, OpenMTRError> {
        self.ctx.http.get_token().await
    }
}
