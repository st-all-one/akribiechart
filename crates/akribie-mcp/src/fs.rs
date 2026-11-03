//! Adaptador de `Fs` para o servidor MCP (mesma semântica do adaptador da CLI).

use std::path::Path;

use akribie_core::error::{Error, Result};
use akribie_core::ports::Fs;

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
        Error::io(format!("falha em `{}`: {error}", path.display()))
    }
}
