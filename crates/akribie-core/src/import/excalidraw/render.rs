//! Renderização das tabelas em `.akrc` canônico.

use super::collect::{Tables, contains, escape, lookup};
use crate::num;

pub(super) fn render(name: &str, tables: &Tables) -> String {
    let Tables {
        nodes,
        groups,
        arrows,
        notes,
    } = tables;
    let mut lines = vec![format!("chart v1 \"{}\" layout=free", escape(name))];
    for group in groups {
        lines.push(format!("group {} \"{}\"", group.id, group.label));
    }
    for node in nodes {
        let inside: Vec<&str> = groups
            .iter()
            .filter(|group| contains(group.rect, node.rect))
            .map(|group| group.id.as_str())
            .collect();
        let membership = if inside.is_empty() {
            String::new()
        } else {
            format!(" in={}", inside.join(","))
        };
        lines.push(format!(
            "node {} \"{}\" shape={} fill={} stroke={} at={},{} size={},{}{membership}",
            node.id,
            node.label,
            node.shape,
            node.fill,
            node.stroke,
            number(node.rect.0),
            number(node.rect.1),
            number(node.rect.2),
            number(node.rect.3)
        ));
    }
    for arrow in arrows {
        let (Some(from), Some(to)) = (arrow.from.as_deref(), arrow.to.as_deref()) else {
            continue;
        };
        let label = if arrow.label.is_empty() {
            String::new()
        } else {
            format!(" \"{}\"", arrow.label)
        };
        lines.push(format!(
            "edge {} -> {}{label} style={}",
            lookup(nodes, from),
            lookup(nodes, to),
            arrow.style
        ));
    }
    for note in notes {
        if note.text.is_empty() {
            continue;
        }
        lines.push(format!(
            "note \"{}\" fill={} at={},{} size={},{}",
            note.text,
            note.fill,
            number(note.rect.0),
            number(note.rect.1),
            number(note.rect.2),
            number(note.rect.3)
        ));
    }
    let mut source = lines.join("\n");
    source.push('\n');
    source
}

pub(super) fn number(value: f32) -> String {
    num::format(value)
}
