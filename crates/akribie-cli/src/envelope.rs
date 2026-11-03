//! Envelope de saída `--json` (ADR-0001 §2.5).
//!
//! Uma linha, `\n` final, campos opcionais omitidos quando ausentes: `data` e `error` nunca
//! aparecem como `null`, e `warnings` aparece sempre (lista vazia quando não há).

use crate::commands::diagnostics_json;
use akribie_core::Error;
use akribie_core::diag::Diagnostic;
use serde::Serialize;
use serde_json::Value;

/// Corpo de erro do envelope.
#[derive(Debug, Serialize)]
pub(crate) struct ErrorBody {
    /// Código textual (`invalid_input`, `not_found`, `io`, `schema`, `internal`).
    pub(crate) code: &'static str,
    /// Mensagem auto-suficiente.
    pub(crate) message: String,
}

/// Envelope `{success, command, data?, error?, warnings}`.
#[derive(Debug, Serialize)]
pub(crate) struct Envelope {
    /// `true` quando o comando concluiu sem erro.
    pub(crate) success: bool,
    /// Nome do verbo (`check`, `render`, …).
    pub(crate) command: &'static str,
    /// Payload do verbo, quando houver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<Value>,
    /// Erro, quando houver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<ErrorBody>,
    /// Avisos no formato `"<code>: <message>"`.
    pub(crate) warnings: Vec<String>,
}

impl Envelope {
    /// Sucesso com payload.
    pub(crate) fn ok(command: &'static str, data: Value) -> Self {
        Self {
            success: true,
            command,
            data: Some(data),
            error: None,
            warnings: Vec::new(),
        }
    }

    /// Falha de processo (sem payload).
    pub(crate) fn failed(command: &'static str, error: &Error) -> Self {
        Self {
            success: false,
            command,
            data: None,
            error: Some(ErrorBody {
                code: error.kind().code(),
                message: error.message().to_string(),
            }),
            warnings: Vec::new(),
        }
    }

    /// Falha de schema, com os diagnósticos preservados em `data`.
    pub(crate) fn failed_with_diagnostics(
        command: &'static str,
        error: &Error,
        diagnostics: &[Diagnostic],
    ) -> Self {
        Self {
            success: false,
            command,
            data: Some(diagnostics_json(diagnostics)),
            error: Some(ErrorBody {
                code: error.kind().code(),
                message: error.message().to_string(),
            }),
            warnings: Vec::new(),
        }
    }

    /// Serializa em uma linha com `\n` final.
    pub(crate) fn render(&self) -> String {
        let mut text = serde_json::to_string(self).unwrap_or_else(|_| {
            "{\"success\":false,\"command\":\"\",\"error\":{\"code\":\"internal\",\"message\":\"falha ao serializar o envelope\"},\"warnings\":[]}".to_string()
        });
        text.push('\n');
        text
    }
}
