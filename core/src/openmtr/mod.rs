//! API pública do openMTR: domínio, cliente, adaptadores e transporte.
//!
//! Reúne os tipos de domínio dos manifestos de transporte de resíduos, o cliente
//! [`crate::openmtr::open_mtr_client::OpenMtrClient`] e seus adaptadores por
//! sistema, além dos pontos de injeção que tornam as operações determinísticas.
/// Adaptadores por sistema (SINIR, SIGOR, MTR-PE) e contexto compartilhado.
pub mod adapters;
/// Cliente HTTP de baixo nível e respostas brutas.
pub mod client;
/// Configuração de conexão: sistema, URL base, token, timeout e limite de RPS.
pub mod config;
/// Tipos de domínio: manifestos, resíduos, partes e filtros.
pub mod domain;
/// Erro público com detalhes RFC 9457 (problem+json).
pub mod error;
/// Envelope de resposta devolvido pelas operações do cliente.
pub mod resposta;
/// Identificação dos sistemas suportados e seleção de adaptadores.
pub mod system;
/// Relógio e gerador de IDs injetáveis para determinismo.
pub mod time_util;
/// Abstração de transporte HTTP, independente de runtime.
pub mod transport;

/// Autenticação clássica (credenciais) para sistemas sem integration token.
pub mod open_mtr_auth;
/// Cliente de alto nível e seu builder, com injeção de relógio e gerador de IDs.
pub mod open_mtr_client;

/// Reexportações dos tipos mais usados da API.
pub mod prelude;
