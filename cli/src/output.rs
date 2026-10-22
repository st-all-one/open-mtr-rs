use serde::Serialize;

pub fn print_json<T: Serialize>(v: &T) {
    if let Ok(s) = serde_json::to_string_pretty(v) {
        println!("{s}");
    }
}
