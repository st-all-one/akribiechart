//! Implementações reais das portas — só a CLI as monta (ADR-0001 §3).
//!
//! Nenhum módulo de `akribie-core` importa `std::fs`, `std::env` ou `std::process::Command`: este
//! arquivo é a fronteira. As demais portas (`Git`, `Env`, `Logger`) ganham adaptador na fase que as
//! usa.

use akribie_core::error::{Error, Result};
use akribie_core::ports::{Fs, Git};
use std::path::Path;

/// Sistema de arquivos real.
pub(crate) struct StdFs;

impl Fs for StdFs {
    fn read(&self, path: &Path) -> Result<String> {
        std::fs::read_to_string(path).map_err(|error| io_error(path, &error))
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        std::fs::read(path).map_err(|error| io_error(path, &error))
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(|error| io_error(parent, &error))?;
        }
        std::fs::write(path, bytes).map_err(|error| io_error(path, &error))
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

fn io_error(path: &Path, error: &std::io::Error) -> Error {
    if error.kind() == std::io::ErrorKind::NotFound {
        Error::not_found(format!("arquivo não encontrado: {}", path.display()))
    } else {
        Error::io(format!("falha de I/O em {}: {error}", path.display()))
    }
}

/// Git real, usado por `--at REV`.
pub(crate) struct StdGit;

impl Git for StdGit {
    fn show(&self, rev: &str, path: &Path) -> Result<String> {
        let output = std::process::Command::new("git")
            .arg("show")
            .arg(format!("{rev}:{}", path.display()))
            .output()
            .map_err(|error| Error::io(format!("falha ao executar git: {error}")))?;
        if !output.status.success() {
            let message = String::from_utf8_lossy(&output.stderr);
            return Err(Error::not_found(format!(
                "`git show {rev}:{}` falhou: {}",
                path.display(),
                message.trim()
            )));
        }
        String::from_utf8(output.stdout)
            .map_err(|_| Error::io(format!("`{}@{rev}` não é UTF-8", path.display())))
    }
}
