//! Adaptador de [`Fs`] sobre o sistema de arquivos real.

use std::path::Path;

use akribie_core::error::Result;
use akribie_core::ports::Fs;

use crate::io_error;

/// Sistema de arquivos real.
pub struct StdFs;

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
