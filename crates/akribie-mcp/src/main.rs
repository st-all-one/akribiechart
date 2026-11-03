//! Servidor MCP do akribie (JSON-RPC 2.0 sobre stdio).
//!
//! Uma mensagem JSON por linha em `stdin`; uma resposta por linha em `stdout`. Comentários/log vão
//! para `stderr`. EOF ou `EPIPE` encerram com exit 0: o cliente fechou, não houve falha.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod base64;
mod fs;
mod protocol;
mod tools;

use std::io::{BufRead, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            return ExitCode::SUCCESS;
        };
        let Some(response) = protocol::handle(&line) else {
            continue;
        };
        if stdout
            .write_all(response.as_bytes())
            .and_then(|()| stdout.write_all(b"\n"))
            .and_then(|()| stdout.flush())
            .is_err()
        {
            // `EPIPE`: o cliente fechou o pipe — sucesso.
            return ExitCode::SUCCESS;
        }
    }
    ExitCode::SUCCESS
}
