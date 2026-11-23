use clap::Subcommand;

#[derive(Subcommand, Debug)]
/// Ações sobre o arquivo de configuração.
pub enum ConfigCmd {
    /// Valida o arquivo de configuração carregado.
    Validate,
    /// Migra o arquivo de configuração para a versão atual.
    Migrate {
        /// Mostra a migração sem gravar alterações no arquivo.
        #[arg(long)]
        dry_run: bool,
    },
}
