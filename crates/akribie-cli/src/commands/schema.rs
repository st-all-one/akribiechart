//! `akr schema`: a gramática em JSON, derivada das tabelas do núcleo.
//!
//! Nada aqui é escrito à mão: statements, chaves, tipos, defaults, valores fechados, tokens de tema,
//! códigos de diagnóstico e exit codes vêm de `syntax`, `theme`, `diag` e `error`. Uma chave nova no
//! núcleo aparece no schema sem edição adicional.

use akribie_core::VERSION;
use akribie_core::diag;
use akribie_core::error::ErrorKind;
use akribie_core::syntax::{self, StmtKind};
use akribie_core::theme::{self, MIN_STROKE_CONTRAST, MIN_TEXT_CONTRAST};
use serde_json::{Value, json};

/// JSON do schema, com quebra de linha final.
pub(crate) fn render() -> String {
    let mut text = serde_json::to_string_pretty(&describe())
        .unwrap_or_else(|_| "{\"schema_version\":1}".to_string());
    text.push('\n');
    text
}

/// Estrutura do schema como valor JSON.
pub(crate) fn describe() -> Value {
    let statements = json!({
        "chart": statement(StmtKind::Chart),
        "group": statement(StmtKind::Group),
        "node": statement(StmtKind::Node),
        "edge": statement(StmtKind::Edge),
        "note": statement(StmtKind::Note),
    });
    let diagnostics: Vec<Value> = diag::ALL
        .iter()
        .map(|code| {
            json!({
                "code": code.code,
                "name": code.name,
                "severity": code.severity.as_str(),
            })
        })
        .collect();
    let exit_codes: Vec<Value> = ErrorKind::ALL
        .iter()
        .map(|kind| {
            json!({
                "exit": kind.exit_code(),
                "code": kind.code(),
                "description": kind.description(),
            })
        })
        .collect();
    let themes: Vec<Value> = syntax::THEMES
        .iter()
        .filter_map(|name| theme::theme(name))
        .map(|theme| {
            let palette: serde_json::Map<String, Value> = theme
                .palette
                .iter()
                .map(|(token, hex)| ((*token).to_string(), json!(hex)))
                .collect();
            json!({
                "name": theme.name,
                "canvas": theme.canvas,
                "ink": theme.ink,
                "palette": palette,
            })
        })
        .collect();
    json!({
        "schema_version": 1,
        "akribie_version": VERSION,
        "keywords": ["chart", "group", "node", "edge", "note"],
        "statements": statements,
        "themes": themes,
        "contrast": {
            "min_text": MIN_TEXT_CONTRAST,
            "min_stroke": MIN_STROKE_CONTRAST,
        },
        "diagnostics": diagnostics,
        "exit_codes": exit_codes,
        "exit_success": 0,
        "exit_fmt_check": 1,
        "exit_diff_changes": 1,
    })
}

fn statement(kind: StmtKind) -> Value {
    let options: Vec<Value> = syntax::option_specs(kind)
        .iter()
        .map(|spec| {
            json!({
                "key": spec.key,
                "type": spec.kind.as_str(),
                "default": spec.default,
                "values": spec.allowed,
                "min": spec.range.map(|(min, _)| min),
                "max": spec.range.map(|(_, max)| max),
            })
        })
        .collect();
    json!({ "options": options })
}
