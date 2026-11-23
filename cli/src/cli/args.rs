use clap::Args;
use std::path::PathBuf;

#[allow(
    clippy::struct_excessive_bools,
    reason = "flags independentes da CLI: escopo, PDF, dedup, retomada e force"
)]
/// Exporta MTRs de contas selecionadas; exige um escopo de conta.
#[derive(Args, Debug)]
pub struct DumpArgs {
    /// Inclui todas as contas habilitadas no dump.
    #[arg(long)]
    pub all_accounts: bool,
    /// ID da conta a incluir; pode ser repetido.
    #[arg(long)]
    pub account: Vec<String>,
    /// Filtra contas pelo slug do sistema (`sinir`, `mtr_pe`, `sigor`).
    #[arg(long)]
    pub system: Option<String>,
    /// Filtra contas por tag; pode ser repetido.
    #[arg(long)]
    pub tag: Vec<String>,
    /// Data inicial do período, no formato `YYYY-MM-DD`.
    #[arg(long)]
    pub from: String,
    /// Data final do período, no formato `YYYY-MM-DD`.
    #[arg(long)]
    pub to: String,
    /// Número de dias por janela de consulta (padrão: 30).
    #[arg(long, default_value = "30")]
    pub window_days: u32,
    /// Diretório de saída dos arquivos gerados (padrão: `dump`).
    #[arg(long, default_value = "dump")]
    pub out: PathBuf,
    /// Baixa também os PDFs dos MTRs.
    #[arg(long)]
    pub pdf: bool,
    /// Mantém duplicatas entre contas em vez de removê-las.
    #[arg(long)]
    pub no_dedup: bool,
    /// Retoma um dump interrompido a partir do progresso salvo.
    #[arg(long)]
    pub resume: bool,
    /// Descarta o estado anterior e refaz o dump do zero.
    #[arg(long)]
    pub force: bool,
    /// Arquivos a gerar: `both`, `csv`, `jsonl` ou `none` (padrão: `both`).
    #[arg(long, default_value = "both")]
    pub format: String,
    /// Número máximo de requisições concorrentes (padrão: 10).
    #[arg(long, default_value = "10")]
    pub concurrency: u32,
}

/// Processa uma lista de operações em JSONL.
#[derive(Args, Debug)]
pub struct BatchArgs {
    /// JSONL de operações; use `-` para stdin.
    #[arg(long, value_name = "JSONL|-")]
    pub file: Option<PathBuf>,
    /// Continua o lote após uma operação falhar.
    #[arg(long)]
    pub continue_on_error: bool,
    /// Valida o lote sem executar as operações.
    #[arg(long)]
    pub dry_run: bool,
}
