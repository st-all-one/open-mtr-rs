//! Subcomandos da CLI, agrupados por área funcional.

mod account;
mod cdf;
mod config;
mod mtr;
mod ref_;
mod self_;

pub use account::AccountCmd;
pub use cdf::CdfCmd;
pub use config::ConfigCmd;
pub use mtr::MtrCmd;
pub use ref_::{RefCmd, RefTable};
pub use self_::SelfCmd;
