fn main() {
    uniffi::generate_scaffolding("./src/open_mtr.udl")
        .expect("Falha ao gerar scaffolding UniFFI");
}
