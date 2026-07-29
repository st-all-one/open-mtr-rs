fn main() {
    uniffi::generate_scaffolding("./src/mtr_sinir.udl")
        .expect("Falha ao gerar scaffolding UniFFI");
}
