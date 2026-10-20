#[allow(
    clippy::expect_used,
    reason = "build script: falha de geração UniFFI deve abortar a compilação"
)]
fn main() {
    uniffi::generate_scaffolding("./src/open_mtr.udl").expect("Falha ao gerar scaffolding UniFFI");
}
