use clap::Parser;
use open_mtr_cli::cli::{CdfCmd, Cli, Command, MtrCmd, RefCmd, RefTable};

fn parse(args: &[&str]) -> Cli {
    Cli::try_parse_from(args).expect("parse")
}

fn mtr_numero(cli: Cli) -> Option<String> {
    match cli.command {
        Command::Mtr {
            sub: MtrCmd::Consultar { numero, .. },
        } => numero,
        _ => None,
    }
}

fn cdf_codigo(cli: Cli) -> Option<String> {
    match cli.command {
        Command::Cdf {
            sub: CdfCmd::Consultar { codigo, .. },
        } => codigo,
        _ => None,
    }
}

fn ref_tabela(cli: Cli) -> Option<RefTable> {
    match cli.command {
        Command::Ref {
            sub: RefCmd::Listar { table, .. },
        } => Some(table),
        _ => None,
    }
}

fn ref_codigo(cli: Cli) -> Option<String> {
    match cli.command {
        Command::Ref {
            sub: RefCmd::PorResiduo { codigo, .. } | RefCmd::PorEstadoFisico { codigo, .. },
        } => Some(codigo),
        _ => None,
    }
}

fn mtr_download_numero(cli: Cli) -> Option<String> {
    match cli.command {
        Command::Mtr {
            sub: MtrCmd::Download { numero, .. },
        } => numero,
        _ => None,
    }
}

fn ref_estado_fisico_codigo(cli: Cli) -> Option<String> {
    match cli.command {
        Command::Ref {
            sub: RefCmd::PorEstadoFisico { codigo, .. },
        } => Some(codigo),
        _ => None,
    }
}

#[test]
fn mtr_consultar_accepts_positional_numero() {
    let cli = parse(&["openmtr", "--login", "x", "mtr", "consultar", "123"]);
    assert_eq!(mtr_numero(cli).as_deref(), Some("123"));
}

#[test]
fn positional_and_params_conflict() {
    let err = Cli::try_parse_from(["openmtr", "mtr", "consultar", "123", "--params", "{}"])
        .expect_err("conflito");
    assert!(err.to_string().contains("cannot be used with"));
}

#[test]
fn cdf_consultar_accepts_positional_codigo() {
    let cli = parse(&["openmtr", "--login", "x", "cdf", "consultar", "42"]);
    assert_eq!(cdf_codigo(cli).as_deref(), Some("42"));
}

#[test]
fn ref_listar_accepts_closed_table_set() {
    let cli = parse(&["openmtr", "--login", "x", "ref", "listar", "estado-fisico"]);
    assert!(matches!(ref_tabela(cli), Some(RefTable::EstadoFisico)));
}

#[test]
fn ref_listar_rejects_unknown_table() {
    let err = Cli::try_parse_from(["openmtr", "ref", "listar", "inexistente"])
        .expect_err("tabela inválida");
    assert!(err.to_string().contains("invalid value"));
}

#[test]
fn ref_por_residuo_takes_positional_codigo() {
    let cli = parse(&["openmtr", "--login", "x", "ref", "por-residuo", "F001"]);
    assert_eq!(ref_codigo(cli).as_deref(), Some("F001"));
}

#[test]
fn write_requires_params() {
    let err = Cli::try_parse_from(["openmtr", "mtr", "criar"]).expect_err("params obrigatório");
    assert!(err.to_string().contains("--params"));
}

#[test]
fn mtr_download_accepts_positional_numero() {
    let cli = parse(&[
        "openmtr", "--login", "x", "mtr", "download", "123", "--out", "out",
    ]);
    assert_eq!(mtr_download_numero(cli).as_deref(), Some("123"));
}

#[test]
fn mtr_download_positional_conflicts_with_params() {
    let err = Cli::try_parse_from([
        "openmtr", "mtr", "download", "123", "--out", "out", "--params", "{}",
    ])
    .expect_err("conflito");
    assert!(err.to_string().contains("cannot be used with"));
}

#[test]
fn cdf_consultar_positional_conflicts_with_params() {
    let err = Cli::try_parse_from(["openmtr", "cdf", "consultar", "42", "--params", "{}"])
        .expect_err("conflito");
    assert!(err.to_string().contains("cannot be used with"));
}

#[test]
fn ref_por_estado_fisico_takes_positional_codigo() {
    let cli = parse(&["openmtr", "--login", "x", "ref", "por-estado-fisico", "EF"]);
    assert_eq!(ref_estado_fisico_codigo(cli).as_deref(), Some("EF"));
}
