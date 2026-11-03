//! Portas de I/O do domínio (ADR-0001 §3).
//!
//! O núcleo nunca toca o sistema operacional: leitura, escrita, `git` e ambiente atravessam estas
//! quatro traits. A CLI monta as implementações reais; os testes usam os fakes de [`fakes`].

use std::path::Path;

use crate::error::Result;

/// Sistema de arquivos.
pub trait Fs {
    /// Lê um arquivo UTF-8.
    ///
    /// # Errors
    /// `NotFound` quando o caminho não existe; `Io` nas demais falhas.
    fn read(&self, path: &Path) -> Result<String>;

    /// Lê um arquivo binário.
    ///
    /// # Errors
    /// `NotFound` quando o caminho não existe; `Io` nas demais falhas.
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>>;

    /// Escreve bytes (cria o arquivo, sobrescreve o conteúdo).
    ///
    /// # Errors
    /// `Io` quando o caminho não puder ser escrito.
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<()>;

    /// `true` quando o caminho existe.
    fn exists(&self, path: &Path) -> bool;
}

/// Consulta de Git usada por `--at REV`.
pub trait Git {
    /// Conteúdo de `path` na revisão `rev` (`git show rev:path`).
    ///
    /// # Errors
    /// `NotFound` quando a revisão ou o caminho não existir; `Io` quando o `git` falhar.
    fn show(&self, rev: &str, path: &Path) -> Result<String>;
}

/// Ambiente do processo (única leitura não determinística, confinada à CLI).
pub trait Env {
    /// Valor da variável, se definida.
    fn var(&self, key: &str) -> Option<String>;
}

/// Saída de log (stderr). `stdout` é reservado a dados.
pub trait Logger {
    /// Mensagem de advertência.
    fn warn(&self, message: &str);
    /// Mensagem informativa.
    fn info(&self, message: &str);
}

pub mod fakes;
