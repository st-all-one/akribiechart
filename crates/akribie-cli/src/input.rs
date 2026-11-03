//! Resolução da entrada: posicional, `-`, stdin ou `--params`.

use akribie_core::Error;
use akribie_core::error::Result;
use akribie_core::ports::Fs;
use std::io::{IsTerminal, Read};
use std::path::Path;

/// Origem do fonte de um verbo.
#[derive(Debug, Clone)]
pub(crate) enum Input {
    /// Caminho de arquivo (lido pela porta `Fs`).
    Path(String),
    /// Texto já em memória (stdin ou `source` de `--params`).
    Text(String),
}

impl Input {
    /// Rótulo usado nos diagnósticos (`arquivo.akrc` ou `<stdin>`).
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Path(path) => path.clone(),
            Self::Text(_) => "<stdin>".to_string(),
        }
    }

    /// Caminho no disco, quando a origem é um arquivo (base para achar `akribie.toml`).
    pub(crate) fn path(&self) -> Option<&Path> {
        match self {
            Self::Path(path) => Some(Path::new(path)),
            Self::Text(_) => None,
        }
    }

    /// Lê o fonte.
    ///
    /// # Errors
    /// `NotFound`/`Io` quando o caminho não puder ser lido.
    pub(crate) fn read(&self, fs: &dyn Fs) -> Result<String> {
        match self {
            Self::Path(path) => fs.read(Path::new(path)),
            Self::Text(text) => Ok(text.clone()),
        }
    }
}

/// Lê todo o stdin como UTF-8.
///
/// # Errors
/// `Io` quando a leitura falhar ou o conteúdo não for UTF-8.
pub(crate) fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|error| Error::io(format!("falha ao ler stdin: {error}")))?;
    Ok(buffer)
}

/// `true` quando o stdin está conectado a um terminal (logo, não há fonte canalizado).
pub(crate) fn stdin_is_terminal() -> bool {
    std::io::stdin().is_terminal()
}

/// Entrada a partir do posicional: `FILE`, `-` ou stdin não-TTY.
///
/// # Errors
/// `InvalidInput` quando não há posicional e o stdin é um terminal.
pub(crate) fn from_positional(file: Option<&str>) -> Result<Input> {
    match file {
        Some("-") => Ok(Input::Text(read_stdin()?)),
        Some(name) => Ok(Input::Path(name.to_string())),
        None if stdin_is_terminal() => Err(Error::invalid_input(
            "nenhuma entrada: informe um arquivo, use `-` ou canalize o fonte (ex.: `akr check -`)",
        )),
        None => Ok(Input::Text(read_stdin()?)),
    }
}

/// Entrada a partir de `--params`: `path` XOR `source`.
///
/// # Errors
/// `InvalidInput` quando ambos ou nenhum forem informados.
pub(crate) fn from_params(path: Option<String>, source: Option<String>) -> Result<Input> {
    match (path, source) {
        (Some(_), Some(_)) => Err(Error::invalid_input(
            "`--params`: use `path` ou `source`, não ambos",
        )),
        (Some(path), None) => Ok(Input::Path(path)),
        (None, Some(text)) => Ok(Input::Text(text)),
        (None, None) => Err(Error::invalid_input(
            "`--params`: informe `path` ou `source`",
        )),
    }
}

/// Conteúdo de `--params`, lendo do stdin quando o valor é `-`.
///
/// # Errors
/// `Io` quando o stdin falhar.
pub(crate) fn read_params(raw: &str) -> Result<String> {
    if raw == "-" {
        read_stdin()
    } else {
        Ok(raw.to_string())
    }
}
