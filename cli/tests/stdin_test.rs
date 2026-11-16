use std::io::Write;
use std::process::{Command, Output, Stdio};

fn openmtr() -> Command {
    Command::new(env!("CARGO_BIN_EXE_openmtr"))
}

fn with_stdin(args: &[&str], stdin: &[u8]) -> Output {
    let mut child = openmtr()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn openmtr");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin)
        .expect("escrever stdin");
    child.wait_with_output().expect("aguardar openmtr")
}

#[test]
fn login_from_stdin_health() {
    let out = with_stdin(
        &["--login", "-", "health"],
        br#"{"system":"sinir","token":"t"}"#,
    );
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("ok"));
}

#[test]
fn batch_from_stdin_health() {
    let out = with_stdin(
        &[
            "--login",
            r#"{"system":"sinir","token":"t"}"#,
            "batch",
            "--file",
            "-",
        ],
        b"{\"op\":\"health\"}\n",
    );
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("\"ok\":true"));
}

#[test]
fn login_stdin_conflict_exits_2() {
    let out = openmtr()
        .args(["--login", "-", "mtr", "consultar", "--params", "-"])
        .stdin(Stdio::null())
        .output()
        .expect("executar openmtr");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("stdin"));
}
