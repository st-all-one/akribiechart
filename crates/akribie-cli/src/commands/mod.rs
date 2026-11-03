//! Comandos da CLI, um arquivo por verbo.

pub(crate) mod check;
pub(crate) mod diff;
pub(crate) mod fmt;
pub(crate) mod format;
pub(crate) mod import;
pub(crate) mod init;
pub(crate) mod prompt;
pub(crate) mod render;
pub(crate) mod schema;

use akribie_core::diag::Diagnostic;
use serde_json::Value;

/// `data.diagnostics[]` no formato de `--json`.
pub(crate) fn diagnostics_json(diagnostics: &[Diagnostic]) -> Value {
    serde_json::json!({
        "diagnostics": diagnostics.iter().map(Diagnostic::to_json).collect::<Vec<_>>(),
    })
}

/// Diagnósticos em texto, um por linha, no formato `<file>:<line>:<col>: <CODE> <name>: …`.
pub(crate) fn render_diagnostics(file: &str, diagnostics: &[Diagnostic]) -> String {
    let mut text = String::new();
    for diagnostic in diagnostics {
        text.push_str(&diagnostic.render(file));
        text.push('\n');
    }
    text
}
