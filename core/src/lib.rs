#![allow(
    clippy::result_large_err,
    reason = "OpenMTRError carrega detalhes RFC 9457; o tamanho é aceito na API pública"
)]

//! Núcleo puro de domínio e cliente do openMTR.
//!
//! Esta crate modela os manifestos de transporte de resíduos (MTR) dos sistemas
//! SINIR, SIGOR e MTR estaduais, junto com o cliente para consultá-los. Não
//! acessa APIs de plataforma diretamente: todo o transporte passa por
//! [`openmtr::transport::HttpTransport`], o que permite reusar a mesma lógica de
//! domínio em native (reqwest), WASM (fetch) ou testes (transportes falsos).
//!
//! O erro público [`openmtr::error::OpenMTRError`] carrega os detalhes RFC 9457
//! (problem+json) devolvidos pelos sistemas. Por isso a crate aceita
//! `clippy::result_large_err`: o tamanho do erro é parte do contrato da API e
//! não é colocado em caixa apenas para satisfazer a lint.

/// Domínio, cliente, adaptadores e infraestrutura do openMTR.
pub mod openmtr;
/// Reexportações convenientes dos tipos mais usados da API.
#[allow(
    clippy::wildcard_imports,
    reason = "reexportação de conveniência da API: o prelude é o contrato público do crate"
)]
pub use openmtr::prelude::*;
