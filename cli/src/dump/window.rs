use super::DumpOptions;
use crate::error::CliError;
use jiff::{ToSpan, civil::Date};

pub(super) fn validate_period(opts: &DumpOptions) -> Result<(), CliError> {
    if opts.from > opts.to {
        return Err(CliError::Usage("--from deve ser <= --to".into()));
    }
    if (opts.to - opts.from).get_days() > 3660 {
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
pub fn parse_date(s: &str) -> Result<Date, CliError> {
    jiff::fmt::strtime::parse("%Y-%m-%d", s)
        .ok()
        .and_then(|p| p.to_date().ok())
        .or_else(|| {
            jiff::fmt::strtime::parse("%d-%m-%Y", s)
                .ok()
                .and_then(|p| p.to_date().ok())
        })
        .ok_or_else(|| CliError::Usage(format!("data inválida: {s} (use YYYY-MM-DD)")))
}

pub(super) fn windows(days: u32, from: Date, to: Date) -> Vec<(Date, Date)> {
    let step = i64::from(days.max(1)) - 1;
    let mut out = Vec::new();
    let mut start = from;
    while start <= to {
        let end = start.checked_add(step.days()).unwrap_or(to).min(to);
        out.push((start, end));
        match end.checked_add(1.days()) {
            Ok(next) => start = next,
            Err(_) => break,
        }
    }
    out
}

pub(crate) fn window_key(start: Date, end: Date) -> String {
    format!("{start}..{end}")
}
