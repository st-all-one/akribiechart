//! Emissão dos resultados dos verbos: texto para pessoas, pipe e envelope `--json`.
//!
//! Dados vão para `stdout`; log e avisos, para `stderr`. Exit codes vêm de `Error::exit_code`
//! (`2` uso, `3` ausente, `5` E/S, `8` schema, `70` interno) ou do próprio verbo (`1` em `fmt
//! --check`, `diff` com mudanças).

pub(crate) mod batch;

use std::process::ExitCode;

use akribie_core::ChangeKind;
use akribie_core::diag::Diagnostic;
use akribie_core::error::{Error, Result};

use crate::commands::check::CheckOutcome;
use crate::commands::diff::DiffOutcome;
use crate::commands::fmt::{Action, FmtOutcome};
use crate::commands::import::ImportOutcome;
use crate::commands::render::RenderOutcome;
use crate::commands::{self, diagnostics_json};
use crate::envelope::Envelope;
use crate::output;

/// Forma de saída escolhida por `--json`.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Mode {
    /// Envelope JSON de uma linha.
    Json,
    /// Texto para consumo humano e para pipe.
    Text,
}

/// Texto de dados: `stdout` com `code`, ou falha de escrita traduzida em exit.
pub(crate) fn emit_text(text: &str, code: u8, command: &'static str) -> ExitCode {
    match output::emit(text, code) {
        Ok(exit) => ExitCode::from(exit),
        Err(error) => fail(Mode::Text, command, &error),
    }
}

/// `check`: diagnósticos (texto) ou envelope com contagem.
pub(crate) fn emit_outcome(outcome: &CheckOutcome, mode: Mode) -> Result<u8> {
    let failed = outcome.failed();
    let code = u8::from(failed).saturating_mul(8);
    if let Mode::Json = mode {
        if failed {
            let message = format!("{} (ver diagnósticos)", outcome.summary());
            let error = Error::schema(message.as_str());
            return output::emit(
                &Envelope::failed_with_diagnostics("check", &error, &outcome.diagnostics).render(),
                code,
            );
        }
        return output::emit(
            &Envelope::ok("check", diagnostics_json(&outcome.diagnostics)).render(),
            0,
        );
    }
    output::emit(
        &commands::render_diagnostics(&outcome.file, &outcome.diagnostics),
        code,
    )
}

/// `import`: a fonte canônica, com perdas em `stderr` ou em `data.diagnostics`.
pub(crate) fn emit_import(outcome: &ImportOutcome, mode: Mode) -> ExitCode {
    if let Mode::Json = mode {
        let data = serde_json::json!({
            "diagnostics": outcome
                .diagnostics
                .iter()
                .map(Diagnostic::to_json)
                .collect::<Vec<_>>(),
            "source": outcome.source,
        });
        return match output::emit(&Envelope::ok("import", data).render(), 0) {
            Ok(exit) => ExitCode::from(exit),
            Err(error) => fail(Mode::Json, "import", &error),
        };
    }
    for warning in &outcome.diagnostics {
        output::warn_line(&warning.render(&outcome.file));
    }
    emit_text(&outcome.source, 0, "import")
}

/// `render`: payload em `stdout` (sem `--out`) ou envelope com `bytes`/`out`.
pub(crate) fn emit_render(outcome: &RenderOutcome, mode: Mode) -> Result<u8> {
    if let Mode::Json = mode {
        if outcome.failed {
            let error = Error::schema("fonte inválida; ver diagnósticos");
            return output::emit(
                &Envelope::failed_with_diagnostics("render", &error, &outcome.diagnostics).render(),
                8,
            );
        }
        let data = serde_json::json!({
            "bytes": outcome.bytes,
            "dry_run": outcome.dry_run,
            "format": outcome.format,
            "height": outcome.height,
            "path": outcome.written,
            "width": outcome.width,
        });
        return output::emit(&Envelope::ok("render", data).render(), 0);
    }
    if outcome.failed {
        return output::emit(
            &commands::render_diagnostics(&outcome.file, &outcome.diagnostics),
            8,
        );
    }
    for warning in &outcome.diagnostics {
        output::warn_line(&warning.render(&outcome.file));
    }
    match (&outcome.payload, &outcome.written) {
        (Some(payload), _) => output::emit(payload, 0),
        (None, Some(path)) if outcome.dry_run => {
            let bytes = outcome.bytes.unwrap_or(0);
            output::emit(&format!("{path}  {bytes} B  (dry-run)\n"), 0)
        }
        (None, _) => Ok(0),
    }
}

/// `diff`: linhas de mudança, exit `1` quando houve diferença.
pub(crate) fn emit_diff(outcome: &DiffOutcome, mode: Mode) -> Result<u8> {
    if outcome.failed() {
        let file = outcome.failed_file.as_deref().unwrap_or("");
        if let Mode::Json = mode {
            let error = Error::schema("fonte inválida; ver diagnósticos");
            return output::emit(
                &Envelope::failed_with_diagnostics("diff", &error, &outcome.diagnostics).render(),
                8,
            );
        }
        return output::emit(&commands::render_diagnostics(file, &outcome.diagnostics), 8);
    }
    for (file, warning) in &outcome.warnings {
        output::warn_line(&warning.render(file));
    }
    let code = u8::from(!outcome.changes.is_empty());
    if let Mode::Json = mode {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();
        for change in &outcome.changes {
            let line = change.line();
            match change.kind {
                ChangeKind::Added => added.push(line),
                ChangeKind::Removed => removed.push(line),
                ChangeKind::Changed => changed.push(line),
            }
        }
        let data = serde_json::json!({
            "added": added,
            "changed": changed,
            "new": outcome.new,
            "old": outcome.old,
            "removed": removed,
        });
        return output::emit(&Envelope::ok("diff", data).render(), code);
    }
    let mut text = String::new();
    for change in &outcome.changes {
        text.push_str(&change.line());
        text.push('\n');
    }
    output::emit(&text, code)
}

/// `fmt`: forma canônica, comparação (`1` quando difere) ou nada após `--write`.
pub(crate) fn emit_fmt(outcome: &FmtOutcome) -> ExitCode {
    match &outcome.action {
        Action::Failed => emit_text(
            &commands::render_diagnostics(&outcome.file, &outcome.diagnostics),
            8,
            "fmt",
        ),
        Action::Emit(formatted) => emit_text(formatted, 0, "fmt"),
        Action::Check {
            formatted,
            original,
        } => {
            if formatted == original {
                ExitCode::SUCCESS
            } else {
                output::warn_line(&format!("{} não está na forma canônica", outcome.file));
                ExitCode::from(1)
            }
        }
        Action::Written => ExitCode::SUCCESS,
    }
}

/// Traduz um erro de processo em saída (envelope ou `stderr`) e exit code.
pub(crate) fn fail(mode: Mode, command: &'static str, error: &Error) -> ExitCode {
    let code = error.exit_code();
    if let Mode::Json = mode {
        return match output::emit(&Envelope::failed(command, error).render(), code) {
            Ok(exit) => ExitCode::from(exit),
            Err(write_error) => {
                output::error_line(write_error.message());
                ExitCode::from(code)
            }
        };
    }
    output::error_line(error.message());
    ExitCode::from(code)
}
