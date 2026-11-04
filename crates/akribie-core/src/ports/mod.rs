//! Portas de I/O do domínio (ADR-0001 §3).
//!
//! O núcleo nunca toca o sistema operacional: leitura, escrita e `git` atravessam estas duas
//! traits. `akribie-adapters` monta as implementações reais (ADR-0006 §3); os testes usam os fakes
//! de [`fakes`]. Um adaptador sem consumidor no domínio não é porta: `Env` e `Logger` existiam sem
//! uso e foram removidas.

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

pub mod fakes;
