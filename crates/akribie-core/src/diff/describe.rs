//! Texto de cada elemento e os campos que o diff compara.

use crate::model::{Chart, Edge, Group, Node, Note};
use crate::num;

use super::{Change, Kind};

pub(super) fn node_identity(node: &Node) -> String {
    format!("node {}", node.id)
}

pub(super) fn group_identity(group: &Group) -> String {
    format!("group {}", group.id)
}

pub(super) fn edge_identity(edge: &Edge) -> String {
    if edge.ordinal == 0 {
        format!("edge {}->{}", edge.from, edge.to)
    } else {
        format!("edge {}->{}#{}", edge.from, edge.to, edge.ordinal)
    }
}

pub(super) fn note_identity(note: &Note) -> String {
    note.near
        .as_deref()
        .map_or_else(|| "note".to_string(), |near| format!("note near={near}"))
}

pub(super) fn node_text(node: &Node) -> String {
    format!(
        "node {} {}",
        node.id,
        quoted(node.declared_label.as_deref())
    )
}

pub(super) fn group_text(group: &Group) -> String {
    format!(
        "group {} {}",
        group.id,
        quoted(group.declared_label.as_deref())
    )
}

pub(super) fn edge_text(edge: &Edge) -> String {
    let label = edge
        .label
        .as_deref()
        .map_or(String::new(), |label| format!(" \"{label}\""));
    if edge.ordinal == 0 {
        format!("edge {}->{}{label}", edge.from, edge.to)
    } else {
        format!("edge {}->{}#{}{label}", edge.from, edge.to, edge.ordinal)
    }
}

pub(super) fn note_text(note: &Note) -> String {
    match &note.near {
        Some(near) => format!("note \"{}\" near={near}", one_line(&note.text)),
        None => format!("note \"{}\"", one_line(&note.text)),
    }
}

fn quoted(label: Option<&str>) -> String {
    label.map_or(String::new(), |label| format!("\"{label}\""))
}

fn one_line(text: &str) -> String {
    text.replace('\n', " ")
}

pub(super) fn option_text(value: Option<&str>) -> String {
    value.unwrap_or("").to_string()
}

pub(super) fn node_fields(node: &Node) -> Vec<(&'static str, String)> {
    let mut fields = vec![
        ("label", node.label.clone()),
        ("shape", node.shape.name().to_string()),
        ("fill", node.fill.name().to_string()),
        ("fillstyle", node.fillstyle.name().to_string()),
        ("stroke", node.stroke.name().to_string()),
        ("style", node.style.name().to_string()),
        ("width", num::format(node.width)),
        ("groups", node.groups.join(",")),
    ];
    if let Some((x, y)) = node.at {
        fields.push(("at", format!("{},{}", num::format(x), num::format(y))));
    }
    if let Some((w, h)) = node.size {
        fields.push(("size", format!("{},{}", num::format(w), num::format(h))));
    }
    fields
}

pub(super) fn group_fields(group: &Group) -> Vec<(&'static str, String)> {
    vec![
        ("label", group.label.clone()),
        ("fill", group.fill.name().to_string()),
        ("stroke", group.stroke.name().to_string()),
        ("style", group.style.name().to_string()),
        ("width", num::format(group.width)),
        ("pad", num::format(group.pad)),
        ("groups", group.groups.join(",")),
    ]
}

pub(super) fn edge_fields(edge: &Edge) -> Vec<(&'static str, String)> {
    vec![
        ("label", edge.label.clone().unwrap_or_default()),
        ("curve", edge.curve.name().to_string()),
        ("head", edge.head.name().to_string()),
        ("tail", edge.tail.name().to_string()),
        ("style", edge.style.name().to_string()),
        ("stroke", edge.stroke.name().to_string()),
        ("width", num::format(edge.width)),
    ]
}

pub(super) fn note_fields(note: &Note) -> Vec<(&'static str, String)> {
    let mut fields = vec![
        ("text", one_line(&note.text)),
        ("fill", note.fill.name().to_string()),
        ("near", note.near.clone().unwrap_or_default()),
    ];
    if let Some((x, y)) = note.at {
        fields.push(("at", format!("{},{}", num::format(x), num::format(y))));
    }
    if let Some((w, h)) = note.size {
        fields.push(("size", format!("{},{}", num::format(w), num::format(h))));
    }
    fields
}

pub(super) fn chart_changes(old: &Chart, new: &Chart, out: &mut Vec<Change>) {
    let fields = [
        (
            "title",
            option_text(old.title.as_deref()),
            option_text(new.title.as_deref()),
        ),
        (
            "layout",
            old.layout.name().to_string(),
            new.layout.name().to_string(),
        ),
        (
            "direction",
            old.direction.name().to_string(),
            new.direction.name().to_string(),
        ),
        (
            "theme",
            old.theme.name.to_string(),
            new.theme.name.to_string(),
        ),
        (
            "fontsize",
            num::format(old.fontsize),
            num::format(new.fontsize),
        ),
        ("pad", num::format(old.pad), num::format(new.pad)),
        ("lang", old.lang.clone(), new.lang.clone()),
        (
            "doc",
            option_text(old.doc.as_deref()),
            option_text(new.doc.as_deref()),
        ),
    ];
    for (field, before, after) in fields {
        if before != after {
            out.push(Change {
                kind: Kind::Changed,
                element: "chart".to_string(),
                field: Some(field),
                before: Some(before),
                after: Some(after),
            });
        }
    }
}
