//! Implementações reais das portas — CLI e servidor MCP as montam (ADR-0006 §3).
//!
//! Nenhum módulo de `akribie-core` importa `std::fs`, `std::env` ou `std::process::Command`: esta
//! crate é a fronteira, e é a única que fala com o sistema operacional. As portas são exatamente
//! as que o domínio usa — [`Fs`](akribie_core::ports::Fs) e [`Git`](akribie_core::ports::Git); um
//! adaptador sem consumidor é superfície morta e não existe.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(not(test), deny(clippy::arithmetic_side_effects, clippy::string_slice))]

use std::path::Path;

use akribie_core::error::Error;

mod fs;
mod git;

pub use fs::StdFs;
pub use git::StdGit;

/// Traduz um erro de I/O em `Error` de domínio, preservando "não encontrado".
fn io_error(path: &Path, error: &std::io::Error) -> Error {
    if error.kind() == std::io::ErrorKind::NotFound {
        Error::not_found(format!("arquivo não encontrado: {}", path.display()))
    } else {
        Error::io(format!("falha de I/O em {}: {error}", path.display()))
    }
}
