//! As três ferramentas MCP: `chart_check`, `chart_render`, `chart_diff`.
//!
//! Toda falha de execução volta como **resultado** com `isError: true` (e a primeira linha do
//! diagnóstico em `content[0].text`); só `tools/call` com nome desconhecido é erro de protocolo.
//! O servidor nunca cai por causa de um `.akrc` ruim.

use std::path::PathBuf;

use akribie_core::config::Config;
use akribie_core::diag::Diagnostic;
use akribie_core::error::Error;
use akribie_core::layout::{self, Placed};
use akribie_core::model::Chart;
use akribie_core::pipeline;
use akribie_core::ports::Fs as _;
use akribie_core::text::Font;
use serde_json::{Value, json};

use akribie_adapters::StdFs;

mod run;

const SOURCE_LABEL: &str = "<source>";

/// `tools/list`: as três ferramentas, com esquema de entrada fechado.
pub(crate) fn list() -> Value {
    json!({
        "tools": [
            {
                "name": "chart_check",
                "description": "Valida uma fonte `.akrc` (inline ou em arquivo) e devolve os diagnósticos.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "source": { "type": "string", "description": "Conteúdo `.akrc`." },
                        "path": { "type": "string", "description": "Caminho de um `.akrc` no disco." },
                        "strict": { "type": "boolean", "description": "Promove aviso a erro." }
                    },
                    "anyOf": [{ "required": ["source"] }, { "required": ["path"] }]
                }
            },
            {
                "name": "chart_render",
                "description": "Compila `.akrc` em SVG/scene/Excalidraw (texto) ou PNG/JPG/AVIF (imagem).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "source": { "type": "string" },
                        "path": { "type": "string" },
                        "format": {
                            "type": "string",
                            "enum": ["svg", "png", "jpg", "avif", "scene", "excalidraw"],
                            "default": "svg"
                        },
                        "scale": { "type": "number", "default": 2.0 }
                    },
                    "anyOf": [{ "required": ["source"] }, { "required": ["path"] }]
                }
            },
            {
                "name": "chart_diff",
                "description": "Compara duas fontes `.akrc` e devolve as mudanças semânticas.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "old_source": { "type": "string" },
                        "new_source": { "type": "string" }
                    },
                    "required": ["old_source", "new_source"]
                }
            }
        ]
    })
}

/// Executa `tools/call`. `Err` apenas para nome de ferramenta desconhecido (erro de protocolo).
pub(crate) fn call(params: &Value) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "`params.name` ausente".to_string())?;
    let empty = json!({});
    let arguments = params.get("arguments").unwrap_or(&empty);
    match name {
        "chart_check" => Ok(run::check(arguments)),
        "chart_render" => Ok(run::render(arguments)),
        "chart_diff" => Ok(run::diff(arguments)),
        other => Err(format!("ferramenta desconhecida: {other}")),
    }
}

struct Source {
    text: String,
    path: Option<PathBuf>,
}

fn source(arguments: &Value) -> Result<Source, String> {
    let inline = arguments.get("source").and_then(Value::as_str);
    let path = arguments.get("path").and_then(Value::as_str);
    match (inline, path) {
        (Some(_), Some(_)) => Err("informe `source` ou `path`, não ambos".to_string()),
        (Some(text), None) => Ok(Source {
            text: text.to_string(),
            path: None,
        }),
        (None, Some(path)) => {
            let path = PathBuf::from(path);
            let text = StdFs
                .read(&path)
                .map_err(|error| error.message().to_string())?;
            Ok(Source {
                text,
                path: Some(path),
            })
        }
        (None, None) => Err("informe `source` ou `path`".to_string()),
    }
}

struct Evaluated {
    chart: Chart,
    placed: Placed,
    diagnostics: Vec<Diagnostic>,
}

fn evaluate(source: &Source) -> Result<Evaluated, String> {
    let fs = StdFs;
    let config = Config::discover(&fs, source.path.as_deref())
        .map_err(|error: Error| error.message().to_string())?;
    let font = Font::load().map_err(|error| error.message().to_string())?;
    let report = pipeline::check(&source.text, &config, &font);
    let placed = layout::layout(&report.chart, &font);
    Ok(Evaluated {
        chart: report.chart,
        placed,
        diagnostics: report.diagnostics,
    })
}

fn text_result(text: &str) -> Value {
    json!({ "content": [{ "type": "text", "text": text }] })
}

fn error_result(text: &str) -> Value {
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": true,
    })
}

/// Texto de erro: primeira linha do primeiro diagnóstico de erro, sem `hint`.
fn diagnostic_line(diagnostics: &[Diagnostic], label: &str) -> String {
    diagnostics
        .iter()
        .find(|diagnostic| diagnostic.is_error())
        .map_or_else(
            || "fonte inválida (sem diagnóstico de erro)".to_string(),
            |diagnostic| {
                diagnostic
                    .render(label)
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .to_string()
            },
        )
}
