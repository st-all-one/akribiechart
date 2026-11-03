//! `scene` JSON: a geometria do desenho posicionado, estável e diffável.
//!
//! O JSON não carrega `path` (`d`): caminho é derivável e ruidoso. O que interessa para depurar é
//! caixa (`rect`), rota em pontos, cores resolvidas, estilos e rótulos. Números passam por
//! [`crate::num::format`] (duas casas, sem zeros) e o objeto é serializado com chaves ordenadas —
//! mesma entrada, mesmos bytes.

use serde_json::{Value, json};

use crate::a11y;
use crate::geometry::{Point, Rect};
use crate::layout::{Placed, PlacedEdge, PlacedGroup, PlacedNode, PlacedNote};
use crate::model::Chart;
use crate::num;
use crate::theme::Theme;

/// Serializa a cena posicionada (JSON bonito, `\n` final).
pub fn scene_json(placed: &Placed, chart: &Chart) -> String {
    let theme = chart.theme;
    let value = json!({
        "canvas": rect_like(placed.width, placed.height),
        "desc": a11y::describe(chart),
        "lang": chart.lang,
        "layout": chart.layout.name(),
        "theme": theme.name,
        "title": chart.title,
        "groups": placed.groups.iter().map(group_value).collect::<Vec<_>>(),
        "edges": placed.edges.iter().map(|edge| edge_value(edge, theme)).collect::<Vec<_>>(),
        "nodes": placed.nodes.iter().map(|node| node_value(node, theme)).collect::<Vec<_>>(),
        "notes": placed.notes.iter().map(|note| note_value(note, theme)).collect::<Vec<_>>(),
        "version": crate::VERSION,
    });
    let mut text = serde_json::to_string_pretty(&value).unwrap_or_default();
    text.push('\n');
    text
}

fn group_value(group: &PlacedGroup) -> Value {
    json!({
        "id": group.id,
        "label": group.label,
        "rect": rect_value(group.rect),
        "stroke": group.stroke.name(),
        "style": group.style.name(),
        "width": number(group.width),
    })
}

fn node_value(node: &PlacedNode, theme: &Theme) -> Value {
    json!({
        "fill": theme.hex(node.fill),
        "fillstyle": node.fillstyle.name(),
        "id": node.id,
        "label": node.label.join(" "),
        "rect": rect_value(node.rect),
        "shape": node.shape.name(),
        "stroke": theme.hex(node.stroke),
        "style": node.style.name(),
        "width": number(node.width),
    })
}

fn edge_value(edge: &PlacedEdge, theme: &Theme) -> Value {
    json!({
        "from": edge.from,
        "head": edge.head,
        "id": format!("{}->{}#{}", edge.from, edge.to, edge.ordinal),
        "label": edge.label,
        "ordinal": edge.ordinal,
        "route": edge.route.iter().map(|point| point_value(*point)).collect::<Vec<_>>(),
        "stroke": theme.hex(edge.stroke),
        "style": edge.style.name(),
        "tail": edge.tail,
        "to": edge.to,
        "width": number(edge.width),
    })
}

fn note_value(note: &PlacedNote, theme: &Theme) -> Value {
    json!({
        "fill": theme.hex(note.fill),
        "rect": rect_value(note.rect),
        "text": note.text.join(" "),
    })
}

fn rect_value(rect: Rect) -> Value {
    json!({
        "h": number(rect.h),
        "w": number(rect.w),
        "x": number(rect.x),
        "y": number(rect.y),
    })
}

fn rect_like(width: f32, height: f32) -> Value {
    json!({ "height": number(height), "width": number(width) })
}

fn point_value(point: Point) -> Value {
    json!([number(point.0), number(point.1)])
}

/// Número como texto canônico, mas como número JSON (`579.12`, não `"579.12"`).
fn number(value: f32) -> Value {
    num::json(value)
}

#[cfg(test)]
mod tests;
