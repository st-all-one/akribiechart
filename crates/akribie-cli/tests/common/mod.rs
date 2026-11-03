//! Harness de teste de integração: executa o binário `akr` de verdade.

#![allow(
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "módulo compartilhado entre crates de teste: cada crate usa um subconjunto, e a asserção precisa falhar alto"
)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Resultado de uma execução do binário.
pub(crate) struct Run {
    /// Exit code (`-1` quando morto por sinal).
    pub code: i32,
    /// `stdout` capturado.
    pub stdout: String,
    /// `stderr` capturado.
    pub stderr: String,
}

/// Executa `akr` com os argumentos dados, opcionalmente alimentando o stdin.
pub(crate) fn run(args: &[&str], stdin: Option<&str>) -> Run {
    let mut command = Command::new(env!("CARGO_BIN_EXE_akr"));
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("spawn akr");
    let mut pipe = child.stdin.take().expect("stdin canalizado");
    if let Some(input) = stdin {
        pipe.write_all(input.as_bytes()).expect("escreve stdin");
    }
    drop(pipe);
    let output = child.wait_with_output().expect("aguarda akr");
    Run {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// Caminho de um golden em `tests/golden/`.
pub(crate) fn golden(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

/// Conteúdo de um golden em `tests/golden/`.
pub(crate) fn read_golden(name: &str) -> String {
    std::fs::read_to_string(golden(name)).expect("lê golden")
}

/// Arquivo temporário exclusivo por processo e nome.
pub(crate) fn temp_file(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("akr-{}-{name}", std::process::id()));
    std::fs::write(&path, content).expect("escreve fixture");
    path
}

/// Executa `akr` com o `stdout` fechado logo após o spawn (cenário de EPIPE).
pub(crate) fn run_with_closed_stdout(args: &[&str]) -> i32 {
    let mut child = Command::new(env!("CARGO_BIN_EXE_akr"))
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn akr");
    drop(child.stdout.take());
    child.wait().expect("aguarda akr").code().unwrap_or(-1)
}

/// Diretório temporário exclusivo por processo e nome, recriado a cada chamada.
pub(crate) fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("akr-{}-{name}", std::process::id()));
    remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("cria diretório de fixture");
    path
}

fn remove_dir_all(path: &Path) {
    drop(std::fs::remove_dir_all(path));
}

/// Executa `akr` com `cwd` explícito (para config, git e linhagem de documentação).
pub(crate) fn run_in(dir: &Path, args: &[&str], stdin: Option<&str>) -> Run {
    let mut command = Command::new(env!("CARGO_BIN_EXE_akr"));
    command
        .current_dir(dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("spawn akr");
    let mut pipe = child.stdin.take().expect("stdin canalizado");
    if let Some(input) = stdin {
        pipe.write_all(input.as_bytes()).expect("escreve stdin");
    }
    drop(pipe);
    let output = child.wait_with_output().expect("aguarda akr");
    Run {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// Executa `git` no diretório, falhando alto se o comando não funcionar.
pub(crate) fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("git disponível");
    assert!(status.success(), "git {args:?} falhou");
}
