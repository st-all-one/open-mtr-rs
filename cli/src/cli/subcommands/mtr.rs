use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
/// Ações sobre MTRs.
pub enum MtrCmd {
    /// Consulta MTRs por número (posicional) ou por filtro em `--params`.
    Consultar {
        /// Número do MTR; exclusivo com `--params`.
        #[arg(value_name = "NUMERO", conflicts_with = "params")]
        numero: Option<String>,
        /// ID da conta a consultar; padrão quando há uma única conta.
        #[arg(long)]
        account: Option<String>,
        /// Filtra contas pelo slug do sistema (`sinir`, `mtr_pe`, `sigor`).
        #[arg(long)]
        system: Option<String>,
        /// Filtra contas por tag; pode ser repetido.
        #[arg(long)]
        tag: Vec<String>,
        /// Consulta em todas as contas habilitadas.
        #[arg(long)]
        all_accounts: bool,
        /// Filtro de consulta em JSON, ou `-` para lê-lo de stdin.
        #[arg(long, value_name = "JSON|-")]
        params: Option<String>,
    },
    /// Cria MTRs a partir de um payload em JSON.
    Criar {
        /// ID da conta; padrão quando há uma única conta.
        #[arg(long)]
        account: Option<String>,
        /// Payload de criação em JSON, ou `-` para stdin (obrigatório).
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
    },
    /// Registra o recebimento de MTRs.
    Receber {
        /// ID da conta; padrão quando há uma única conta.
        #[arg(long)]
        account: Option<String>,
        /// Payload de recebimento em JSON, ou `-` para stdin (obrigatório).
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
    },
    /// Cancela MTRs existentes.
    Cancelar {
        /// ID da conta; padrão quando há uma única conta.
        #[arg(long)]
        account: Option<String>,
        /// Payload de cancelamento em JSON, ou `-` para stdin (obrigatório).
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
    },
    /// Modifica MTRs existentes.
    Modificar {
        /// ID da conta; padrão quando há uma única conta.
        #[arg(long)]
        account: Option<String>,
        /// Payload de modificação em JSON, ou `-` para stdin (obrigatório).
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
    },
    /// Aceita MTRs recebidos.
    Aceitar {
        /// ID da conta; padrão quando há uma única conta.
        #[arg(long)]
        account: Option<String>,
        /// Payload de aceite em JSON, ou `-` para stdin (obrigatório).
        #[arg(long, value_name = "JSON|-", required = true)]
        params: String,
    },
    /// Baixa PDFs de um ou mais números.
    Download {
        /// Número do MTR; exclusivo com `--params`.
        #[arg(value_name = "NUMERO", conflicts_with = "params")]
        numero: Option<String>,
        /// ID da conta; padrão quando há uma única conta.
        #[arg(long)]
        account: Option<String>,
        /// Números ou filtro em JSON, ou `-` para stdin.
        #[arg(long, value_name = "JSON|-")]
        params: Option<String>,
        /// Diretório onde salvar os PDFs baixados.
        #[arg(long)]
        out: PathBuf,
    },
}
