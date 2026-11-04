//! Fechamento do desenho: grupos, notas, arestas e normalização do canvas.

use indexmap::IndexMap;

use crate::geometry::{self, Rect};
use crate::model::{Chart, Curve};
use crate::text::Gauge;

use super::{
    Placed, PlacedEdge, PlacedGroup, PlacedNode, PlacedNote, arrows, label_lines, labels, notes,
};

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
    let mut groups = group_boxes(chart, font, &nodes);
    let mut note_list = notes::note_boxes(chart, font, &nodes, &index);
    let mut edges = edge_routes(chart, &nodes, &index);
    labels::place(chart, font, &nodes, &groups, &mut edges);
    let (width, height) = normalize(
        &mut nodes,
        &mut groups,
        &mut note_list,
        &mut edges,
        chart.pad,
    );
    Placed {
        width,
        height,
        groups,
        nodes,
        edges,
        notes: note_list,
    }
}

/// Caixa de cada grupo: união recursiva dos nós e subgrupos, inflada por `pad`.
fn group_boxes(chart: &Chart, gauge: &Gauge, nodes: &[PlacedNode]) -> Vec<PlacedGroup> {
    let count = chart.groups.len();
    // O rótulo é medido uma vez: a faixa da caixa e as linhas desenhadas saem do mesmo cálculo.
    let mut metrics: Vec<(Vec<String>, f32, f32)> = chart
        .groups
        .iter()
        .map(|group| labels::group_metrics(gauge, &group.label, chart.fontsize))
        .collect();
    let mut memo: Vec<Option<Rect>> = vec![None; count];
    let mut visiting = vec![false; count];
    for position in 0..count {
        resolve_group(position, chart, &metrics, nodes, &mut memo, &mut visiting);
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
                label_lines: metrics
                    .get_mut(position)
                    .map_or_else(Vec::new, |metric| std::mem::take(&mut metric.0)),
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
    metrics: &[(Vec<String>, f32, f32)],
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
            && let Some(child_rect) =
                resolve_group(child_position, chart, metrics, nodes, memo, visiting)
        {
            rect = Some(rect.map_or(child_rect, |current| current.union(child_rect)));
        }
    }
    let (text_w, band) = metrics
        .get(position)
        .map_or((0.0, 0.0), |(_, text_w, band)| (*text_w, *band));
    let result = rect.map(|box_| {
        let padded = box_.inflate(group.pad);
        // A faixa do título sai da largura do grupo, mas nunca fica mais estreita que o texto:
        // um rótulo longo alarga a caixa em vez de transbordar.
        Rect::new(
            padded.x,
            padded.y - band,
            padded.w.max(text_w),
            padded.h + band,
        )
    });
    if let Some(slot) = memo.get_mut(position) {
        *slot = result;
    }
    if let Some(flag) = visiting.get_mut(position) {
        *flag = false;
    }
    result
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
                label_rect: None,
                label_lines: Vec::new(),
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
        if let Some(rect) = edge.label_rect {
            observe((rect.x, rect.y));
            observe((rect.right(), rect.bottom()));
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
        if let Some(rect) = edge.label_rect {
            edge.label_rect = Some(rect.translate(dx, dy));
        }
    }
    (max.0 - min.0 + 2.0 * pad, max.1 - min.1 + 2.0 * pad)
}
