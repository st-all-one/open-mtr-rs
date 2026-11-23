use crate::cli::RefTable;

#[test]
fn ref_table_accepts_cli_canonical_name() {
    use clap::ValueEnum;
    assert!(matches!(
        RefTable::from_str("estado-fisico", true),
        Ok(RefTable::EstadoFisico)
    ));
}
