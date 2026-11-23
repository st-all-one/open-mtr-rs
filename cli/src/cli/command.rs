use clap::Subcommand;

use super::{AccountCmd, BatchArgs, CdfCmd, ConfigCmd, DumpArgs, MtrCmd, RefCmd, SelfCmd};

#[derive(Subcommand, Debug)]
/// Comandos principais disponíveis na CLI.
pub enum Command {
    /// Gerencia as contas configuradas.
    Account {
        /// Subcomando de contas.
        #[command(subcommand)]
        sub: AccountCmd,
    },
    /// Valida e migra o arquivo de configuração.
    Config {
        /// Subcomando de configuração.
        #[command(subcommand)]
        sub: ConfigCmd,
    },
    /// Consulta e altera MTRs.
    Mtr {
        /// Subcomando de MTRs.
        #[command(subcommand)]
        sub: MtrCmd,
    },
    /// Consulta, gera e baixa CDFs.
    Cdf {
        /// Subcomando de CDFs.
        #[command(subcommand)]
        sub: CdfCmd,
    },
    /// Consulta tabelas de referência do sistema.
    Ref {
        /// Subcomando de tabelas de referência.
        #[command(subcommand)]
        sub: RefCmd,
    },
    /// Verifica o binário sem exigir configuração nem chamar a API.
    Health,
    /// Exporta MTRs de várias contas para arquivos locais.
    Dump(DumpArgs),
    /// Executa um lote de operações lidas de um JSONL.
    Batch(BatchArgs),
    /// Mostra a versão ou atualiza o próprio binário.
    #[command(name = "self")]
    SelfTool {
        /// Subcomando de automação do binário.
        #[command(subcommand)]
        sub: SelfCmd,
    },
}
