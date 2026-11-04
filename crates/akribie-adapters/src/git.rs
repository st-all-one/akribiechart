//! Adaptador de [`Git`] para `--at REV` (`git show REV:FILE`).

use std::path::Path;

use akribie_core::error::{Error, Result};
use akribie_core::ports::Git;

/// Git real, usado por `--at REV`.
pub struct StdGit;

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
