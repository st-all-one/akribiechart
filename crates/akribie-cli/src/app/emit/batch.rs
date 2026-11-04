//! Emissão dos lotes (`--batch`): uma linha por item em texto, envelope com `items` em `--json`.

use std::fmt::Write as _;
use std::process::ExitCode;

use akribie_core::Error;
use akribie_core::diag::Diagnostic;

use super::{Mode, emit_text, fail};
use crate::commands::check::batch::BatchCheckOutcome;
use crate::commands::render::batch::BatchRenderOutcome;
use crate::envelope::Envelope;
use crate::output;

/// Anexa `args` a `text`; escrever em `String` nunca falha.
fn append(text: &mut String, args: std::fmt::Arguments<'_>) {
    text.write_fmt(args).unwrap_or(());
}

/// `check --batch`: uma linha por item (texto) ou envelope com `items`.
pub(crate) fn emit_batch_check(outcomes: &[BatchCheckOutcome], mode: Mode) -> ExitCode {
    let ok = outcomes.iter().filter(|outcome| outcome.ok()).count();
    let failed = outcomes.len().saturating_sub(ok);
    let all_failed = ok == 0;
    let code = u8::from(all_failed).saturating_mul(8);
    if let Mode::Json = mode {
        let items: Vec<serde_json::Value> = outcomes
            .iter()
            .map(|outcome| {
                let mut item = serde_json::json!({
                    "diagnostics": outcome
                        .diagnostics
                        .iter()
                        .map(Diagnostic::to_json)
                        .collect::<Vec<_>>(),
                    "index": outcome.index,
                    "name": outcome.name,
                    "ok": outcome.ok(),
                });
                if let Some((code, message)) = outcome.failure()
                    && let Some(map) = item.as_object_mut()
                {
                    map.insert(
                        "error".to_string(),
                        serde_json::json!({ "code": code, "message": message }),
                    );
                }
                item
            })
            .collect();
        let data = serde_json::json!({ "failed": failed, "items": items, "ok": ok });
        let envelope = if all_failed {
            let error = Error::schema("todos os itens do lote falharam");
            let mut envelope = Envelope::failed_with_diagnostics("check", &error, &[]);
            envelope.data = Some(data);
            envelope
        } else {
            Envelope::ok("check", data)
        };
        return match output::emit(&envelope.render(), code) {
            Ok(exit) => ExitCode::from(exit),
            Err(error) => fail(Mode::Json, "check", &error),
        };
    }
    let mut text = String::new();
    for outcome in outcomes {
        if outcome.ok() {
            append(
                &mut text,
                format_args!(
                    "{}  ok  {} aviso(s)\n",
                    outcome.name,
                    outcome.warning_count()
                ),
            );
        } else {
            let (code, message) = outcome
                .failure()
                .unwrap_or(("schema", "item inválido".to_string()));
            append(
                &mut text,
                format_args!("{}  erro  {code} {message}\n", outcome.name),
            );
        }
    }
    emit_text(&text, code, "check")
}

/// `render --batch`: `<alvo>  ok  <bytes> B` / `<alvo>  erro  <CODE> <message>`.
pub(crate) fn emit_batch_render(outcomes: &[BatchRenderOutcome], mode: Mode) -> ExitCode {
    let ok = outcomes.iter().filter(|outcome| outcome.ok()).count();
    let failed = outcomes.len().saturating_sub(ok);
    let all_failed = ok == 0;
    let code = u8::from(all_failed).saturating_mul(8);
    if let Mode::Json = mode {
        let items: Vec<serde_json::Value> = outcomes
            .iter()
            .map(|outcome| match outcome.failure() {
                Some((code, message)) => serde_json::json!({
                    "error": { "code": code, "message": message },
                    "index": outcome.index,
                    "name": outcome.name,
                    "ok": false,
                }),
                None => serde_json::json!({
                    "bytes": outcome.bytes,
                    "index": outcome.index,
                    "name": outcome.name,
                    "ok": true,
                    "out": outcome.out,
                }),
            })
            .collect();
        let data = serde_json::json!({ "failed": failed, "items": items, "ok": ok });
        let envelope = if all_failed {
            let error = Error::schema("todos os itens do lote falharam");
            let mut envelope = Envelope::failed_with_diagnostics("render", &error, &[]);
            envelope.data = Some(data);
            envelope
        } else {
            Envelope::ok("render", data)
        };
        return match output::emit(&envelope.render(), code) {
            Ok(exit) => ExitCode::from(exit),
            Err(error) => fail(Mode::Json, "render", &error),
        };
    }
    let mut text = String::new();
    for outcome in outcomes {
        if outcome.ok() {
            append(
                &mut text,
                format_args!(
                    "{}  ok  {} B\n",
                    outcome.target(),
                    outcome.bytes.unwrap_or(0)
                ),
            );
        } else {
            let (code, message) = outcome
                .failure()
                .unwrap_or(("schema", "item inválido".to_string()));
            append(
                &mut text,
                format_args!("{}  erro  {code} {message}\n", outcome.target()),
            );
        }
    }
    emit_text(&text, code, "render")
}
