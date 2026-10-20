#![allow(
    clippy::result_large_err,
    reason = "OpenMTRError carrega detalhes RFC 9457; o tamanho é aceito na API pública"
)]

pub mod openmtr;
pub use openmtr::prelude::*;
