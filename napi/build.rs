//! Script de build do binding napi-rs (emite as flags de linker do addon).

extern crate napi_build;
fn main() {
    napi_build::setup();
}
