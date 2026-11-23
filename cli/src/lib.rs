//! CLI utilitária multi-conta do openMTR.
//!
//! O binário `openmtr` é uma casca fina: o comportamento vive nos módulos
//! públicos abaixo, exercitáveis por testes de integração.

#![allow(
    clippy::cargo_common_metadata,
    reason = "crate interna do workspace, publish = false"
)]

/// Seleção, validação e teste de contas configuradas.
pub mod account;
/// Execução de lotes de operações em JSONL.
pub mod batch;
/// Definição da interface de linha de comando e despacho dos subcomandos.
pub mod cli;
/// Carregamento e validação do arquivo de configuração multi-conta.
pub mod config;
mod download;
/// Exportação de MTRs em janelas, com deduplicação e retomada.
pub mod dump;
/// Tipos de erro e códigos de saída da CLI.
pub mod error;
pub mod login;
/// Migração de versões do arquivo de configuração.
pub mod migrate;
/// Normalização de dados de entrada (datas, números e textos).
pub mod normalize;
/// Formatação da saída em texto ou JSON de máquina.
pub mod output;
/// Interpretação de `--params` e `--login` como JSON ou stdin.
pub mod params;
/// Construção do runtime a partir de configuração ou login explícito.
pub mod runtime;
/// Consulta, verificação e aplicação de atualizações do binário.
pub mod selfupdate;
pub mod session;
