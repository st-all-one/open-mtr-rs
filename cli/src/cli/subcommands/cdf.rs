use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
/// Ações sobre CDFs.
pub enum CdfCmd {
    /// Consulta CDFs por código (posicional) ou por filtro em `--params`.
    Consultar {
        /// Código do CDF; exclusivo com `--params`.
        #[arg(value_name = "CODIGO", conflicts_with = "params")]
        codigo: Option<String>,
        /// ID da conta; padrão quando há uma única conta.
        #[arg(long)]
        account: Option<String>,
        /// Filtro de consulta em JSON, ou `-` para lê-lo de stdin.
        #[arg(long, value_name = "JSON|-")]
        params: Option<String>,
    },
    /// Gera CDFs a partir de um payload em JSON.
    Gerar {
        /// ID da conta; padrão quando há uma única conta.
        #[arg(long)]
        account: Option<String>,
        /// Payload de geração em JSON, ou `-` para stdin (obrigatório).
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
    },
    /// Baixa os PDFs dos CDFs pedidos no payload.
    Download {
        /// ID da conta; padrão quando há uma única conta.
        #[arg(long)]
        account: Option<String>,
        /// Payload com os CDFs em JSON, ou `-` para stdin (obrigatório).
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
        /// Diretório onde salvar os PDFs baixados.
        #[arg(long)]
        out: PathBuf,
    },
}
