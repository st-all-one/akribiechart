//! Execução das três ferramentas: avaliar, formatar a saída, nunca derrubar o servidor.

use akribie_core::diag::has_errors;
use akribie_core::emit::{self, RasterFormat};
use akribie_core::paint::paint;
use serde_json::{Value, json};

use super::{SOURCE_LABEL, Source, diagnostic_line, error_result, evaluate, source, text_result};
use crate::base64;

pub(super) fn check(arguments: &Value) -> Value {
    let source = match source(arguments) {
        Ok(source) => source,
        Err(message) => return error_result(&message),
    };
    let strict = arguments
        .get("strict")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let evaluated = match evaluate(&source) {
        Ok(evaluated) => evaluated,
        Err(message) => return error_result(&message),
    };
    let failed =
        has_errors(&evaluated.diagnostics) || (strict && !evaluated.diagnostics.is_empty());
    if failed {
        return error_result(&diagnostic_line(&evaluated.diagnostics, SOURCE_LABEL));
    }
    let mut text = format!(
        "ok ({} erro(s), {} aviso(s))",
        0,
        evaluated.diagnostics.len()
    );
    for warning in &evaluated.diagnostics {
        text.push('\n');
        text.push_str(&warning.render(SOURCE_LABEL));
    }
    text_result(&text)
}

pub(super) fn render(arguments: &Value) -> Value {
    let source = match source(arguments) {
        Ok(source) => source,
        Err(message) => return error_result(&message),
    };
    let format = arguments
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or("svg");
    let scale = arguments
        .get("scale")
        .and_then(Value::as_f64)
        .unwrap_or(2.0);
    let evaluated = match evaluate(&source) {
        Ok(evaluated) => evaluated,
        Err(message) => return error_result(&message),
    };
    if has_errors(&evaluated.diagnostics) {
        return error_result(&diagnostic_line(&evaluated.diagnostics, SOURCE_LABEL));
    }
    let theme = evaluated.chart.theme;
    match format {
        "svg" => {
            let scene = paint(&evaluated.chart, &evaluated.placed);
            text_result(&emit::svg(&scene, theme))
        }
        "scene" => text_result(&emit::scene_json(&evaluated.placed, &evaluated.chart)),
        "excalidraw" => text_result(&emit::excalidraw(&evaluated.chart, &evaluated.placed)),
        other => {
            let Some(raster) = RasterFormat::from_name(other) else {
                return error_result(&format!("formato desconhecido: {other}"));
            };
            let scene = paint(&evaluated.chart, &evaluated.placed);
            let document = emit::svg(&scene, theme);
            match emit::encode(&document, raster, f32_scale(scale)) {
                Ok(bytes) => image_result(&bytes, mime(raster)),
                Err(error) => error_result(error.message()),
            }
        }
    }
}

pub(super) fn diff(arguments: &Value) -> Value {
    let old = arguments
        .get("old_source")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let new = arguments
        .get("new_source")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let mut charts = Vec::new();
    for text in [old, new] {
        let source = Source {
            text: text.to_string(),
            path: None,
        };
        match evaluate(&source) {
            Ok(evaluated) if !has_errors(&evaluated.diagnostics) => charts.push(evaluated.chart),
            Ok(evaluated) => {
                return error_result(&diagnostic_line(&evaluated.diagnostics, "<source>"));
            }
            Err(message) => return error_result(&message),
        }
    }
    let (Some(before), Some(after)) = (charts.first(), charts.get(1)) else {
        return error_result("fonte ausente");
    };
    let changes = akribie_core::diff(before, after);
    if changes.is_empty() {
        return text_result("sem mudanças (0)");
    }
    let mut text = String::new();
    for change in &changes {
        text.push_str(&change.line());
        text.push('\n');
    }
    text_result(text.trim_end())
}

fn mime(raster: RasterFormat) -> &'static str {
    match raster {
        RasterFormat::Png => "image/png",
        RasterFormat::Jpeg => "image/jpeg",
        RasterFormat::Avif => "image/avif",
    }
}

fn image_result(bytes: &[u8], mime: &str) -> Value {
    json!({
        "content": [{ "type": "image", "data": base64::encode(bytes), "mimeType": mime }],
    })
}

/// `f64` de JSON → `f32` de escala (valores de diagrama cabem com folga).
fn f32_scale(value: f64) -> f32 {
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "escala de rasterização é pequena e positiva"
    )]
    let scale = value as f32;
    scale
}
