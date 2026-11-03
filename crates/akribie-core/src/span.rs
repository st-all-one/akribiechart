//! Posição em um arquivo-fonte.

use serde::Serialize;

/// Posição de um token ou diagnóstico: linha e coluna **1-based**, coluna em caracteres
/// (não em bytes), para que a mensagem aponte o caractere que o autor vê.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    /// Linha, começando em 1.
    pub line: u32,
    /// Coluna em caracteres, começando em 1.
    pub col: u32,
}

impl Span {
    /// Cria uma posição.
    pub const fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }

    /// Posição no início da linha.
    pub const fn line_start(line: u32) -> Self {
        Self { line, col: 1 }
    }
}
