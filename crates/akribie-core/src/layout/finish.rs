//! Fechamento do desenho: grupos, notas, arestas e normalização do canvas.

use indexmap::IndexMap;

use crate::geometry::{self, Rect};
use crate::model::{Chart, Curve};
use crate::text::{self, Gauge};

use super::{Placed, PlacedEdge, PlacedGroup, PlacedNode, PlacedNote, arrows, label_lines};

/// Distância entre um nó e a nota ancorada nele.
const NOTE_GAP: f32 = 24.0;
/// Respiro interno de uma nota.
const NOTE_PADDING: f32 = 8.0;

pub(super) fn assemble(
    chart: &Chart,
    font: &Gauge,
    sizes: &[(f32, f32)],
    rects: &[Rect],
) -> Placed {
    let mut nodes: Vec<PlacedNode> = chart
        .nodes
        .iter()
        .enumerate()
        .zip(sizes.iter())
        .filter_map(|((index, node), size)| {
            let rect = rects.get(index)?;
            Some(PlacedNode {
                id: node.id.clone(),
                rect: *rect,
                shape: node.shape,
                label: label_lines(&node.label, node.shape, *size, chart.fontsize, font),
                fill: node.fill,
                fillstyle: node.fillstyle,
                stroke: node.stroke,
                style: node.style,
                width: node.width,
                span: node.span,
            })
        })
        .collect();
    let index: IndexMap<&str, usize> = chart
        .nodes
        .iter()
        .enumerate()
        .map(|(position, node)| (node.id.as_str(), position))
        .collect();
    let mut groups = group_boxes(chart, &nodes);
    let mut notes = note_boxes(chart, font, &nodes, &index);
    let mut edges = edge_routes(chart, &nodes, &index);
    let (width, height) = normalize(&mut nodes, &mut groups, &mut notes, &mut edges, chart.pad);
    Placed {
        width,
        height,
        groups,
        nodes,
        edges,
        notes,
    }
}

/// Caixa de cada grupo: união recursiva dos nós e subgrupos, inflada por `pad`.
fn group_boxes(chart: &Chart, nodes: &[PlacedNode]) -> Vec<PlacedGroup> {
    let count = chart.groups.len();
    let mut memo: Vec<Option<Rect>> = vec![None; count];
    let mut visiting = vec![false; count];
    for position in 0..count {
        resolve_group(position, chart, nodes, &mut memo, &mut visiting);
    }
    chart
        .groups
        .iter()
        .enumerate()
        .filter_map(|(position, group)| {
            let rect = memo.get(position).copied().flatten()?;
            Some(PlacedGroup {
                id: group.id.clone(),
                rect,
                label: group.label.clone(),
                stroke: group.stroke,
                style: group.style,
                width: group.width,
                span: group.span,
            })
        })
        .collect()
}

fn resolve_group(
    position: usize,
    chart: &Chart,
    nodes: &[PlacedNode],
    memo: &mut Vec<Option<Rect>>,
    visiting: &mut Vec<bool>,
) -> Option<Rect> {
    if let Some(rect) = memo.get(position).copied().flatten() {
        return Some(rect);
    }
    let group = chart.groups.get(position)?;
    if *visiting.get(position).unwrap_or(&true) {
        return None;
    }
    if let Some(flag) = visiting.get_mut(position) {
        *flag = true;
    }
    let mut rect: Option<Rect> = None;
    for (index, model) in chart.nodes.iter().enumerate() {
        if model.groups.iter().any(|id| id == &group.id)
            && let Some(node) = nodes.get(index)
        {
            rect = Some(rect.map_or(node.rect, |current| current.union(node.rect)));
        }
    }
    for (child_position, child) in chart.groups.iter().enumerate() {
        if child.groups.iter().any(|id| id == &group.id)
            && let Some(child_rect) = resolve_group(child_position, chart, nodes, memo, visiting)
        {
            rect = Some(rect.map_or(child_rect, |current| current.union(child_rect)));
        }
    }
    let result = rect.map(|box_| {
        let padded = box_.inflate(group.pad);
        let band = chart.fontsize * text::LINE_HEIGHT;
        Rect::new(padded.x, padded.y - band, padded.w, padded.h + band)
    });
    if let Some(slot) = memo.get_mut(position) {
        *slot = result;
    }
    if let Some(flag) = visiting.get_mut(position) {
        *flag = false;
    }
    result
}

/// Notas: `at` manda; sem ele, a nota fica na diagonal inferior direita do nó de `near=`
/// (abaixo do rank e à direita da coluna, sem disputar com os dois).
fn note_boxes(
    chart: &Chart,
    font: &Gauge,
    nodes: &[PlacedNode],
    index: &IndexMap<&str, usize>,
) -> Vec<PlacedNote> {
    chart
        .notes
        .iter()
        .map(|note| {
            let lines = font.wrap(&note.text, chart.fontsize, text::MAX_LABEL_W);
            let (width, height) = note.size.unwrap_or_else(|| {
                let (text_w, text_h) = font.label_size(&note.text, chart.fontsize);
                (text_w + 2.0 * NOTE_PADDING, text_h + 2.0 * NOTE_PADDING)
            });
            let anchor = note
                .near
                .as_ref()
                .and_then(|id| index.get(id.as_str()))
                .and_then(|position| nodes.get(*position));
            let rect = match (note.at, anchor) {
                (Some(at), _) => Rect::new(at.0, at.1, width, height),
                (None, Some(target)) => Rect::new(
                    target.rect.right() + NOTE_GAP,
                    target.rect.bottom() + NOTE_GAP * 0.5,
                    width,
                    height,
                ),
                (None, None) => Rect::new(0.0, 0.0, width, height),
            };
            PlacedNote {
                rect,
                text: lines,
                fill: note.fill,
                span: note.span,
            }
        })
        .collect()
}

/// Arestas: rota do contorno de origem ao contorno de destino.
fn edge_routes(
    chart: &Chart,
    nodes: &[PlacedNode],
    index: &IndexMap<&str, usize>,
) -> Vec<PlacedEdge> {
    chart
        .edges
        .iter()
        .filter_map(|edge| {
            let from = index
                .get(edge.from.as_str())
                .and_then(|position| nodes.get(*position))?;
            let to = index
                .get(edge.to.as_str())
                .and_then(|position| nodes.get(*position))?;
            let route = if edge.from == edge.to {
                geometry::self_edge(from.rect)
            } else {
                let start = geometry::outline_hit(from.rect, from.shape, to.rect.center());
                let end = geometry::outline_hit(to.rect, to.shape, from.rect.center());
                match edge.curve {
                    Curve::Line => geometry::route_line(start, end),
                    Curve::Arc => geometry::route_arc(start, end),
                    Curve::Elbow => geometry::route_elbow(start, end),
                }
            };
            let middle =
                geometry::route_midpoint(&route).map_or((0.0, 0.0), |(point, _direction)| point);
            let (head, tail) = arrows(edge.head, edge.tail);
            Some(PlacedEdge {
                from: edge.from.clone(),
                to: edge.to.clone(),
                ordinal: edge.ordinal,
                curve: edge.curve,
                label: edge.label.clone(),
                label_at: middle,
                route,
                head,
                tail,
                stroke: edge.stroke,
                style: edge.style,
                width: edge.width,
                span: edge.span,
            })
        })
        .collect()
}

/// Translada tudo para que o canto superior esquerdo fique em `(pad, pad)`.
fn normalize(
    nodes: &mut [PlacedNode],
    groups: &mut [PlacedGroup],
    notes: &mut [PlacedNote],
    edges: &mut [PlacedEdge],
    pad: f32,
) -> (f32, f32) {
    let mut min = (f32::INFINITY, f32::INFINITY);
    let mut max = (f32::NEG_INFINITY, f32::NEG_INFINITY);
    let mut observe = |point: geometry::Point| {
        min = (min.0.min(point.0), min.1.min(point.1));
        max = (max.0.max(point.0), max.1.max(point.1));
    };
    for node in &*nodes {
        observe((node.rect.x, node.rect.y));
        observe((node.rect.right(), node.rect.bottom()));
    }
    for group in &*groups {
        observe((group.rect.x, group.rect.y));
        observe((group.rect.right(), group.rect.bottom()));
    }
    for note in &*notes {
        observe((note.rect.x, note.rect.y));
        observe((note.rect.right(), note.rect.bottom()));
    }
    for edge in &*edges {
        for point in &edge.route {
            observe(*point);
        }
    }
    if !min.0.is_finite() || !min.1.is_finite() {
        return (2.0 * pad, 2.0 * pad);
    }
    let dx = pad - min.0;
    let dy = pad - min.1;
    for node in &mut *nodes {
        node.rect = node.rect.translate(dx, dy);
    }
    for group in &mut *groups {
        group.rect = group.rect.translate(dx, dy);
    }
    for note in &mut *notes {
        note.rect = note.rect.translate(dx, dy);
    }
    for edge in &mut *edges {
        for point in &mut edge.route {
            *point = (point.0 + dx, point.1 + dy);
        }
        edge.label_at = (edge.label_at.0 + dx, edge.label_at.1 + dy);
    }
    (max.0 - min.0 + 2.0 * pad, max.1 - min.1 + 2.0 * pad)
}
