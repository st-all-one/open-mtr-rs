use clap::Subcommand;

#[derive(Subcommand, Debug)]
/// Ações de consulta e atualização do próprio binário.
pub enum SelfCmd {
    /// Exibe a versão instalada do `openmtr`.
    Version,
    /// Verifica ou aplica atualizações do próprio binário.
    Update {
        /// Apenas verifica se existe versão mais recente.
        #[arg(long)]
        check: bool,
        /// Mostra o plano de atualização sem aplicá-lo.
        #[arg(long)]
        dry_run: bool,
        /// Versão específica a instalar (tag do release).
        #[arg(long)]
        version: Option<String>,
        /// Força a atualização mesmo sem versão mais recente.
        #[arg(long)]
        force: bool,
    },
}
