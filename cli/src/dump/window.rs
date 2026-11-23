use super::DumpOptions;
use crate::error::CliError;
use chrono::NaiveDate;

pub(super) fn validate_period(opts: &DumpOptions) -> Result<(), CliError> {
    if opts.from > opts.to {
        return Err(CliError::Usage("--from deve ser <= --to".into()));
    }
    if (opts.to - opts.from).num_days() > 3660 {
        return Err(CliError::Usage("período maior que 3660 dias".into()));
    }
    if opts.window_days == 0 {
        return Err(CliError::Usage("--window-days deve ser >= 1".into()));
    }
    match opts.format.as_str() {
        "jsonl" | "csv" | "both" | "none" => Ok(()),
        other => Err(CliError::Usage(format!("--format inválido: {other}"))),
    }
}

/// Analisa uma data `YYYY-MM-DD` ou `DD-MM-YYYY`, com erro de uso se inválida.
pub fn parse_date(s: &str) -> Result<NaiveDate, CliError> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(s, "%d-%m-%Y"))
        .map_err(|_| CliError::Usage(format!("data inválida: {s} (use YYYY-MM-DD)")))
}

pub(super) fn windows(days: u32, from: NaiveDate, to: NaiveDate) -> Vec<(NaiveDate, NaiveDate)> {
    let step = chrono::Duration::days(i64::from(days.max(1)) - 1);
    let mut out = Vec::new();
    let mut start = from;
    while start <= to {
        let end = (start + step).min(to);
        out.push((start, end));
        start = end + chrono::Duration::days(1);
    }
    out
}

pub(crate) fn window_key(start: NaiveDate, end: NaiveDate) -> String {
    format!("{start}..{end}")
}
