//! Caminhos seguros para arquivos baixados.

use std::path::{Path, PathBuf};

/// Reduz `filename` ao basename e o junta a `out`.
///
/// Nomes com `../`, separadores ou vazios nunca escapam do diretório.
pub(crate) fn safe_path(out: &Path, filename: &str) -> PathBuf {
    let name = filename.rsplit(['/', '\\']).next().unwrap_or("arquivo");
    let name = if name.is_empty() || name == "." || name == ".." {
        "arquivo"
    } else {
        name
    };
    out.join(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_basename_and_blocks_escape() {
        let out = Path::new("/tmp/out");
        assert_eq!(safe_path(out, "123.pdf"), out.join("123.pdf"));
        assert_eq!(safe_path(out, "../../etc/passwd"), out.join("passwd"));
        assert_eq!(safe_path(out, "/etc/passwd"), out.join("passwd"));
        assert_eq!(safe_path(out, "..\\..\\x.pdf"), out.join("x.pdf"));
        assert_eq!(safe_path(out, ""), out.join("arquivo"));
        assert_eq!(safe_path(out, ".."), out.join("arquivo"));
    }
}
