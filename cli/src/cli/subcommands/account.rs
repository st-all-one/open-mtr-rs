use clap::Subcommand;

#[derive(Subcommand, Debug)]
/// Ações sobre contas configuradas.
pub enum AccountCmd {
    /// Lista as contas habilitadas (id, sistema, acesso e tags).
    List,
    /// Testa uma conta configurada pelo ID.
    Test {
        /// ID da conta a testar.
        id: String,
    },
}
