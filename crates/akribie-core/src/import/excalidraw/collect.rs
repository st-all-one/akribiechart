//! Classificação dos elementos em tabelas e decodificação de cor/estilo.

use serde_json::Value;
use std::collections::BTreeMap;

use crate::diag::{Diagnostic, W005, W006};
use crate::span::Span;
use crate::theme::PAPER;

pub(super) struct Tables {
    pub(super) nodes: Vec<Row>,
    pub(super) groups: Vec<GroupRow>,
    pub(super) arrows: Vec<ArrowRow>,
    pub(super) notes: Vec<NoteRow>,
}

/// Classifica os elementos (grupo/nota por `customData`, o resto por tipo) em tabelas.
pub(super) fn collect(elements: &[&Value]) -> (Vec<Diagnostic>, Tables) {
    let mut diagnostics = Vec::new();
    let labels = labels(elements);
    let mut tables = Tables {
        nodes: Vec::new(),
        groups: Vec::new(),
        arrows: Vec::new(),
        notes: Vec::new(),
    };
    for element in elements {
        let kind = element.get("type").and_then(Value::as_str).unwrap_or("");
        let hint = element.pointer("/customData/akr").and_then(Value::as_str);
        let external = original_id(element);
        let label = labels.get(&external).cloned().unwrap_or_default();
        match hint {
            Some("group") => {
                tables.groups.push(GroupRow {
                    id: sanitize(&external),
                    label,
                    rect: rect_of(element),
                });
                continue;
            }
            Some("note") => {
                let fill = color_token(element, "backgroundColor", "none", &mut diagnostics);
                tables.notes.push(NoteRow {
                    text: label,
                    fill,
                    rect: rect_of(element),
                });
                continue;
            }
            Some("edge") => {
                tables.arrows.push(arrow_row(element, &label));
                continue;
            }
            _ => {}
        }
        match kind {
            "rectangle" | "ellipse" | "diamond" => {
                let shape = match kind {
                    "ellipse" => "circle",
                    "diamond" => "diamond",
                    _ => "rounded",
                };
                let fill = color_token(element, "backgroundColor", "none", &mut diagnostics);
                let stroke = color_token(element, "strokeColor", "muted", &mut diagnostics);
                let id = sanitize(&external);
                tables.nodes.push(Row {
                    external,
                    id,
                    label,
                    shape,
                    fill,
                    stroke,
                    rect: rect_of(element),
                });
            }
            "text" if element.get("containerId").and_then(Value::as_str).is_none() => {
                let text = element
                    .get("text")
                    .or_else(|| element.get("originalText"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let fill = color_token(element, "backgroundColor", "none", &mut diagnostics);
                tables.notes.push(NoteRow {
                    text: escape(text),
                    fill,
                    rect: rect_of(element),
                });
            }
            "arrow" => tables.arrows.push(arrow_row(element, &label)),
            "text" => {}
            other => diagnostics.push(dropped(element, &format!("tipo `{other}`"))),
        }
    }
    (diagnostics, tables)
}

pub(super) struct Row {
    pub(super) external: String,
    pub(super) id: String,
    pub(super) label: String,
    pub(super) shape: &'static str,
    pub(super) fill: String,
    pub(super) stroke: String,
    pub(super) rect: (f32, f32, f32, f32),
}

pub(super) struct GroupRow {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) rect: (f32, f32, f32, f32),
}

pub(super) struct NoteRow {
    pub(super) text: String,
    pub(super) fill: String,
    pub(super) rect: (f32, f32, f32, f32),
}

pub(super) struct ArrowRow {
    pub(super) from: Option<String>,
    pub(super) to: Option<String>,
    pub(super) label: String,
    pub(super) style: &'static str,
}

pub(super) fn arrow_row(element: &Value, label: &str) -> ArrowRow {
    ArrowRow {
        from: element
            .pointer("/startBinding/elementId")
            .and_then(Value::as_str)
            .map(str::to_string),
        to: element
            .pointer("/endBinding/elementId")
            .and_then(Value::as_str)
            .map(str::to_string),
        label: label.to_string(),
        style: stroke_style(element),
    }
}

pub(super) fn rect_of(element: &Value) -> (f32, f32, f32, f32) {
    (
        coordinate(element, "x"),
        coordinate(element, "y"),
        coordinate(element, "width").max(1.0),
        coordinate(element, "height").max(1.0),
    )
}

pub(super) fn coordinate(element: &Value, key: &str) -> f32 {
    let Some(value) = element.get(key).and_then(Value::as_f64) else {
        return 0.0;
    };
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "coordenadas de diagrama cabem em f32"
    )]
    let value = value as f32;
    value
}

pub(super) fn contains(group: (f32, f32, f32, f32), element: (f32, f32, f32, f32)) -> bool {
    let cx = element.0 + element.2 * 0.5;
    let cy = element.1 + element.3 * 0.5;
    cx >= group.0 && cx <= group.0 + group.2 && cy >= group.1 && cy <= group.1 + group.3
}

/// Rótulos por container (texto cujo `containerId` aponta para o elemento).
pub(super) fn labels(elements: &[&Value]) -> BTreeMap<String, String> {
    let mut labels = BTreeMap::new();
    for element in elements {
        if element.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }
        let Some(container) = element.get("containerId").and_then(Value::as_str) else {
            continue;
        };
        let text = element
            .get("text")
            .or_else(|| element.get("originalText"))
            .and_then(Value::as_str)
            .unwrap_or("");
        labels.insert(container.to_string(), escape(text));
    }
    labels
}

pub(super) fn original_id(element: &Value) -> String {
    element
        .get("id")
        .and_then(Value::as_str)
        .map_or_else(|| "sem-id".to_string(), str::to_string)
}

pub(super) fn lookup(nodes: &[Row], external: &str) -> String {
    nodes
        .iter()
        .find(|node| node.external == external)
        .map_or_else(|| sanitize(external), |node| node.id.clone())
}

pub(super) fn sanitize(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() || out.starts_with(|ch: char| ch.is_ascii_digit()) {
        out.insert(0, 'n');
    }
    out
}

pub(super) fn color_token(
    element: &Value,
    key: &str,
    fallback: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> String {
    let raw = element
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("transparent");
    if raw.eq_ignore_ascii_case("transparent") || raw.is_empty() {
        return fallback.to_string();
    }
    if let Some((name, _)) = PAPER
        .palette
        .iter()
        .find(|(_, hex)| hex.eq_ignore_ascii_case(raw))
    {
        return (*name).to_string();
    }
    {
        {
            diagnostics.push(
                Diagnostic::new(
                    W005,
                    Span::new(1, 1),
                    format!("cor `{raw}` sem token de tema; usando `{fallback}`"),
                )
                .with_hint("tokens aceitos: paper, ink, muted, blue, green, amber, red, violet"),
            );
            fallback.to_string()
        }
    }
}

pub(super) fn dropped(element: &Value, why: &str) -> Diagnostic {
    let id = element.get("id").and_then(Value::as_str).unwrap_or("?");
    Diagnostic::new(
        W006,
        Span::new(1, 1),
        format!("elemento `{id}` descartado ({why})"),
    )
    .with_hint("o `.akrc` não representa imagens, frames nem agrupamentos do Excalidraw")
}

pub(super) fn stroke_style(element: &Value) -> &'static str {
    match element
        .get("strokeStyle")
        .and_then(Value::as_str)
        .unwrap_or("solid")
    {
        "dashed" => "dashed",
        "dotted" => "dotted",
        _ => "solid",
    }
}

pub(super) fn escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}
