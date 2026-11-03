//! Implementações em memória das portas, para testes determinísticos.
//!
//! Nenhum teste do domínio toca o sistema de arquivos real: `MemFs`/`FakeGit` dão o mesmo resultado
//! em toda máquina, e `MemFs` devolve `NotFound`/`Io` com as mesmas mensagens do adaptador real.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::{Env, Fs, Git, Logger};
use crate::error::{Error, Result};

/// Sistema de arquivos em memória, ordenado (`BTreeMap`) para iteração determinística.
#[derive(Debug, Default)]
pub struct MemFs {
    files: Mutex<BTreeMap<PathBuf, Vec<u8>>>,
}

impl MemFs {
    /// Cria um sistema de arquivos vazio.
    pub fn new() -> Self {
        Self::default()
    }

    /// Acrescenta (ou substitui) um arquivo.
    pub fn with_file(mut self, path: impl Into<PathBuf>, content: impl Into<Vec<u8>>) -> Self {
        if let Ok(files) = self.files.get_mut() {
            files.insert(path.into(), content.into());
        }
        self
    }

    /// Conteúdo textual de um arquivo, se existir e for UTF-8.
    pub fn contents(&self, path: impl AsRef<Path>) -> Option<String> {
        let files = self.files.lock().ok()?;
        files
            .get(path.as_ref())
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
    }

    /// Caminhos registrados, em ordem.
    pub fn paths(&self) -> Vec<PathBuf> {
        self.files
            .lock()
            .map_or_else(|_| Vec::new(), |files| files.keys().cloned().collect())
    }
}

impl Fs for MemFs {
    fn read(&self, path: &Path) -> Result<String> {
        let bytes = self.read_bytes(path)?;
        String::from_utf8(bytes)
            .map_err(|_| Error::io(format!("{} não é UTF-8 válido", path.display())))
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        let files = self
            .files
            .lock()
            .map_err(|_| Error::internal("MemFs: lock envenenado"))?;
        files
            .get(path)
            .cloned()
            .ok_or_else(|| Error::not_found(format!("arquivo não encontrado: {}", path.display())))
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        let mut files = self
            .files
            .lock()
            .map_err(|_| Error::internal("MemFs: lock envenenado"))?;
        files.insert(path.to_path_buf(), bytes.to_vec());
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        self.files
            .lock()
            .is_ok_and(|files| files.contains_key(path))
    }
}

/// Git em memória: conteúdo por `(revisão, caminho)`.
#[derive(Debug, Default)]
pub struct FakeGit {
    revisions: BTreeMap<(String, PathBuf), String>,
}

impl FakeGit {
    /// Cria um git sem revisões.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra o conteúdo de `path` em `rev`.
    pub fn with_revision(
        mut self,
        rev: &str,
        path: impl Into<PathBuf>,
        content: impl Into<String>,
    ) -> Self {
        self.revisions
            .insert((rev.to_string(), path.into()), content.into());
        self
    }
}

impl Git for FakeGit {
    fn show(&self, rev: &str, path: &Path) -> Result<String> {
        self.revisions
            .get(&(rev.to_string(), path.to_path_buf()))
            .cloned()
            .ok_or_else(|| {
                Error::not_found(format!("{rev}:{} não existe na revisão", path.display()))
            })
    }
}

/// Ambiente em memória.
#[derive(Debug, Default)]
pub struct FakeEnv {
    vars: BTreeMap<String, String>,
}

impl FakeEnv {
    /// Cria um ambiente vazio.
    pub fn new() -> Self {
        Self::default()
    }

    /// Define uma variável.
    pub fn with_var(mut self, key: &str, value: &str) -> Self {
        self.vars.insert(key.to_string(), value.to_string());
        self
    }
}

impl Env for FakeEnv {
    fn var(&self, key: &str) -> Option<String> {
        self.vars.get(key).cloned()
    }
}

/// Logger que grava as mensagens em ordem, para asserção em teste.
#[derive(Debug, Default)]
pub struct RecordingLogger {
    entries: Mutex<Vec<String>>,
}

impl RecordingLogger {
    /// Cria um logger vazio.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mensagens gravadas, em ordem.
    pub fn entries(&self) -> Vec<String> {
        self.entries
            .lock()
            .map_or_else(|_| Vec::new(), |entries| entries.clone())
    }

    fn push(&self, level: &str, message: &str) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.push(format!("{level}: {message}"));
        }
    }
}

impl Logger for RecordingLogger {
    fn warn(&self, message: &str) {
        self.push("warn", message);
    }

    fn info(&self, message: &str) {
        self.push("info", message);
    }
}

#[cfg(test)]
mod tests {
    use super::{FakeGit, Git, MemFs};
    use crate::ports::Fs;
    use std::path::Path;

    #[test]
    fn memfs_roundtrip_and_not_found() {
        let fs = MemFs::new().with_file("a.akrc", "chart v1\n");
        assert_eq!(
            fs.read(Path::new("a.akrc")).ok().as_deref(),
            Some("chart v1\n")
        );
        assert!(fs.exists(Path::new("a.akrc")));
        assert!(!fs.exists(Path::new("b.akrc")));
        assert_eq!(
            fs.read(Path::new("b.akrc"))
                .err()
                .map(|error| error.exit_code()),
            Some(3)
        );
        fs.write(Path::new("out.svg"), b"<svg/>").ok();
        assert_eq!(fs.contents("out.svg").as_deref(), Some("<svg/>"));
    }

    #[test]
    fn fake_git_show() {
        let git = FakeGit::new().with_revision("HEAD", "a.akrc", "chart v1\n");
        assert_eq!(
            git.show("HEAD", Path::new("a.akrc")).ok().as_deref(),
            Some("chart v1\n")
        );
        assert!(git.show("v0", Path::new("a.akrc")).is_err());
    }
}
