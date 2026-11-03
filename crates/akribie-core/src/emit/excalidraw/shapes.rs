//! Grupos, nós e notas como elementos Excalidraw.

use serde_json::{Value, json};

use super::support::{
    base, color, element_id, excalidraw_kind, fill_style, set, stroke_style, tag, text_element,
};
use crate::geometry::Rect;
use crate::layout::{PlacedGroup, PlacedNode, PlacedNote};
use crate::model::{FontFamily, Shape};
use crate::num;
use crate::sketch::seed_for;
use crate::theme::Theme;

pub(super) fn push_group(
    elements: &mut Vec<Value>,
    group: &PlacedGroup,
    theme: &Theme,
    family: FontFamily,
) {
    let id = element_id(&group.id);
    let mut shape = base(
        &id,
        "rectangle",
        group.rect,
        color(theme, group.stroke),
        "transparent",
        seed_for(&group.id),
    );
    set(
        &mut shape,
        "strokeStyle",
        Value::from(stroke_style(group.style)),
    );
    set(&mut shape, "strokeWidth", num::json(group.width));
    set(
        &mut shape,
        "boundElements",
        json!([{ "id": format!("{id}-label"), "type": "text" }]),
    );
    tag(&mut shape, "group");
    elements.push(shape);
    let label = Rect::new(
        group.rect.x + 8.0,
        group.rect.y + 4.0,
        group.rect.w - 16.0,
        (group.rect.h * 0.2).max(20.0),
    );
    elements.push(text_element(
        &format!("{id}-label"),
        &group.label,
        label,
        theme.ink,
        16.0,
        Some(&id),
        family,
    ));
}

pub(super) fn push_node(
    elements: &mut Vec<Value>,
    node: &PlacedNode,
    theme: &Theme,
    fontsize: f32,
    family: FontFamily,
) {
    let id = element_id(&node.id);
    let mut shape = base(
        &id,
        excalidraw_kind(node.shape),
        node.rect,
        color(theme, node.stroke),
        color(theme, node.fill),
        seed_for(&node.id),
    );
    set(
        &mut shape,
        "fillStyle",
        Value::from(fill_style(node.fillstyle.name())),
    );
    set(
        &mut shape,
        "strokeStyle",
        Value::from(stroke_style(node.style)),
    );
    set(&mut shape, "strokeWidth", num::json(node.width));
    if node.shape == Shape::Rounded {
        set(&mut shape, "roundness", json!({ "type": 3 }));
    }
    let text_id = format!("{id}-text");
    set(
        &mut shape,
        "boundElements",
        json!([{ "id": text_id, "type": "text" }]),
    );
    tag(&mut shape, "node");
    elements.push(shape);
    elements.push(text_element(
        &text_id,
        &node.label.join("\n"),
        node.rect,
        theme.ink,
        fontsize,
        Some(&id),
        family,
    ));
}

pub(super) fn push_note(
    elements: &mut Vec<Value>,
    note: &PlacedNote,
    theme: &Theme,
    fontsize: f32,
    family: FontFamily,
) {
    let id = format!("akr-note-{}", note.span.line);
    let mut shape = base(
        &id,
        "rectangle",
        note.rect,
        theme.muted,
        color(theme, note.fill),
        seed_for(&id),
    );
    set(&mut shape, "fillStyle", Value::from("solid"));
    tag(&mut shape, "note");
    let text_id = format!("{id}-text");
    set(
        &mut shape,
        "boundElements",
        json!([{ "id": text_id, "type": "text" }]),
    );
    elements.push(shape);
    elements.push(text_element(
        &text_id,
        &note.text.join("\n"),
        note.rect,
        theme.ink,
        fontsize,
        Some(&id),
        family,
    ));
}
