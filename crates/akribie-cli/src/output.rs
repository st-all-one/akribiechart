//! Saída padrão: escrita de dados com tratamento de EPIPE.

use akribie_core::error::{Error, Result};
use std::io::Write;

/// Escreve bytes em `stdout`.
///
/// Devolve `false` quando o consumidor fechou o pipe (`EPIPE`): para `akr`, isso é sucesso — o
/// comando produziu a saída que alguém pediu e parou de olhar (ADR-0001 §2.5).
pub(crate) fn write_bytes(bytes: &[u8]) -> Result<bool> {
    let mut stdout = std::io::stdout().lock();
    match stdout.write_all(bytes).and_then(|()| stdout.flush()) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => Ok(false),
        Err(error) => Err(Error::io(format!("falha ao escrever em stdout: {error}"))),
    }
}

/// Escreve texto em `stdout`.
pub(crate) fn write_stdout(text: &str) -> Result<bool> {
    write_bytes(text.as_bytes())
}

/// Emite texto e devolve o exit code: `0` quando houve EPIPE, `code` caso contrário.
pub(crate) fn emit(text: &str, code: u8) -> Result<u8> {
    if write_stdout(text)? { Ok(code) } else { Ok(0) }
}

/// Escreve uma linha de erro em `stderr` (log/uso, nunca dados).
pub(crate) fn error_line(message: &str) {
    let mut stderr = std::io::stderr().lock();
    if writeln!(stderr, "erro: {message}").is_err() {
        // stderr indisponível: não há para onde reportar.
    }
}

/// Escreve uma linha de aviso em `stderr` (não é erro de processo).
pub(crate) fn warn_line(message: &str) {
    let mut stderr = std::io::stderr().lock();
    if writeln!(stderr, "aviso: {message}").is_err() {
        // stderr indisponível: não há para onde reportar.
    }
}
