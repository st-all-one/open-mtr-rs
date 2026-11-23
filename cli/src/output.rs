use serde::Serialize;

/// Formato de emissão dos comandos (`--json` vs. texto).
#[derive(Clone, Copy, Debug, Default)]
pub enum OutputFormat {
    /// JSON legível por máquina (via [`print_json`]).
    Json,
    /// Texto/JSON compacto na saída padrão.
    #[default]
    Text,
}

/// Emite `v` como JSON pretty-printed na saída padrão; falhas de serialização são silenciosas.
#[allow(clippy::print_stdout, reason = "CLI: a saída padrão é o produto")]
pub fn print_json<T: Serialize>(v: &T) {
    if let Ok(s) = serde_json::to_string_pretty(v) {
        println!("{s}");
    }
}
