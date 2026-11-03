//! Erros de processo (não de schema) e os exit codes do binário.
//!
//! A correspondência entre variante e exit code vive **aqui** e em nenhum outro lugar: a CLI
//! devolve [`Error::exit_code`], e `akr schema` publica a mesma tabela.

use std::fmt;

/// Resultado do domínio.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Natureza do erro, mapeada 1:1 em exit code.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Uso inválido: flag, formato, stdin ausente, lote vazio.
    InvalidInput,
    /// Arquivo, revisão git ou diretório inexistente.
    NotFound,
    /// Falha de leitura/escrita.
    Io,
    /// `.akrc` inválido (≥1 diagnóstico de erro) ou lote 100% falho.
    Schema,
    /// Invariante interna quebrada.
    Internal,
}

impl ErrorKind {
    /// Todas as variantes, na ordem dos exit codes.
    pub const ALL: [Self; 5] = [
        Self::InvalidInput,
        Self::NotFound,
        Self::Io,
        Self::Schema,
        Self::Internal,
    ];

    /// Exit code associado (tabela do ADR-0001 §2.5).
    pub const fn exit_code(self) -> u8 {
        match self {
            Self::InvalidInput => 2,
            Self::NotFound => 3,
            Self::Io => 5,
            Self::Schema => 8,
            Self::Internal => 70,
        }
    }

    /// Código textual usado no envelope `--json`.
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::NotFound => "not_found",
            Self::Io => "io",
            Self::Schema => "schema",
            Self::Internal => "internal",
        }
    }

    /// Descrição curta para `akr schema`.
    pub const fn description(self) -> &'static str {
        match self {
            Self::InvalidInput => "uso inválido: flag, formato, stdin ausente, lote vazio",
            Self::NotFound => "arquivo, revisão git ou diretório inexistente",
            Self::Io => "falha de leitura/escrita",
            Self::Schema => "fonte inválida (≥1 erro) ou lote 100% falho",
            Self::Internal => "invariante interna quebrada",
        }
    }
}

/// Erro do domínio: natureza + mensagem auto-suficiente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    /// Cria um erro.
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// Uso inválido.
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidInput, message)
    }

    /// Recurso inexistente.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, message)
    }

    /// Falha de I/O.
    pub fn io(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Io, message)
    }

    /// Fonte inválida.
    pub fn schema(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Schema, message)
    }

    /// Invariante interna.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }

    /// Natureza do erro.
    pub const fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Mensagem, sem prefixo.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Exit code correspondente.
    pub const fn exit_code(&self) -> u8 {
        self.kind.exit_code()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for Error {}
