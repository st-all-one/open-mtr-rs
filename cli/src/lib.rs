//! CLI utilitária multi-conta do openMTR.
//!
//! O binário `openmtr` é uma casca fina: o comportamento vive nos módulos
//! públicos abaixo, exercitáveis por testes de integração.

#![allow(
    clippy::cargo_common_metadata,
    reason = "crate interna do workspace, publish = false"
)]

pub mod account;
pub mod batch;
pub mod cli;
pub mod config;
mod download;
pub mod dump;
pub mod error;
pub mod login;
pub mod migrate;
pub mod normalize;
pub mod output;
pub mod params;
pub mod runtime;
pub mod selfupdate;
pub mod session;
