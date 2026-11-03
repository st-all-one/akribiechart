//! `layout=mindmap`: radial. A raiz é o primeiro nó sem aresta de entrada; `direction` é ignorado.

use indexmap::IndexMap;

use crate::geometry::Rect;
use crate::model::Chart;

/// Distância entre anéis.
const RADIUS_STEP: f32 = 180.0;

pub(super) fn run(chart: &Chart, sizes: &[(f32, f32)]) -> Vec<Rect> {
    let count = sizes.len();
    if count == 0 {
        return Vec::new();
    }
    let (children, root) = spanning(chart, count);
    let mut weight = vec![1.0_f32; count];
    weigh(root, &children, &mut weight);
    let mut centers = vec![(0.0_f32, 0.0_f32); count];
    spread(
        root,
        0,
        (0.0, std::f32::consts::TAU),
        &children,
        &weight,
        &mut centers,
    );
    (0..count)
        .map(|index| {
            let (width, height) = sizes.get(index).copied().unwrap_or((0.0, 0.0));
            let (x, y) = centers.get(index).copied().unwrap_or((0.0, 0.0));
            Rect::new(x - width * 0.5, y - height * 0.5, width, height)
        })
        .collect()
}

/// Peso da subárvore: 1 para folha, soma dos filhos caso contrário.
fn weigh(index: usize, children: &[Vec<usize>], weight: &mut [f32]) {
    let list = children.get(index).cloned().unwrap_or_default();
    if list.is_empty() {
        return;
    }
    let mut total = 0.0_f32;
    for child in list {
        weigh(child, children, weight);
        total += weight.get(child).copied().unwrap_or(1.0);
    }
    if let Some(slot) = weight.get_mut(index) {
        *slot = total;
    }
}

fn spread(
    index: usize,
    depth: usize,
    span: (f32, f32),
    children: &[Vec<usize>],
    weight: &[f32],
    centers: &mut [(f32, f32)],
) {
    let (start, end) = span;
    let middle = f32::midpoint(start, end);
    let radius = RADIUS_STEP * ring(depth);
    if let Some(slot) = centers.get_mut(index) {
        *slot = (radius * middle.cos(), radius * middle.sin());
    }
    let list = children.get(index).cloned().unwrap_or_default();
    let total: f32 = list
        .iter()
        .map(|child| weight.get(*child).copied().unwrap_or(1.0))
        .sum();
    if total <= 0.0 {
        return;
    }
    let mut cursor = start;
    for child in list {
        let share = weight.get(child).copied().unwrap_or(1.0) / total;
        let child_span = (cursor, cursor + (end - start) * share);
        spread(child, depth + 1, child_span, children, weight, centers);
        cursor = child_span.1;
    }
}

/// Nasce em `f32` sem `as`: profundidade cabe em `u16` por construção (limite de nós).
fn ring(depth: usize) -> f32 {
    f32::from(u16::try_from(depth).unwrap_or(u16::MAX))
}

/// Árvore geradora em BFS: ignora arestas que voltariam a um nó já visitado.
fn spanning(chart: &Chart, count: usize) -> (Vec<Vec<usize>>, usize) {
    let index: IndexMap<&str, usize> = chart
        .nodes
        .iter()
        .enumerate()
        .map(|(position, node)| (node.id.as_str(), position))
        .collect();
    let mut outgoing: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut has_parent = vec![false; count];
    for edge in &chart.edges {
        let (Some(&from), Some(&to)) = (index.get(edge.from.as_str()), index.get(edge.to.as_str()))
        else {
            continue;
        };
        if from == to {
            continue;
        }
        let Some(list) = outgoing.get_mut(from) else {
            continue;
        };
        if list.contains(&to) {
            continue;
        }
        list.push(to);
        if let Some(flag) = has_parent.get_mut(to) {
            *flag = true;
        }
    }
    let root = (0..count)
        .find(|position| !has_parent.get(*position).copied().unwrap_or(false))
        .unwrap_or(0);
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut visited = vec![false; count];
    let mut queue = vec![root];
    if let Some(flag) = visited.get_mut(root) {
        *flag = true;
    }
    let mut cursor = 0;
    while cursor < queue.len() {
        let current = queue.get(cursor).copied().unwrap_or(0);
        cursor += 1;
        for child in outgoing.get(current).cloned().unwrap_or_default() {
            if visited.get(child).copied().unwrap_or(true) {
                continue;
            }
            if let Some(flag) = visited.get_mut(child) {
                *flag = true;
            }
            if let Some(list) = children.get_mut(current) {
                list.push(child);
            }
            queue.push(child);
        }
    }
    // Nós inalcançáveis viram filhos da raiz: o desenho nunca perde um nó.
    for position in 0..count {
        if !visited.get(position).copied().unwrap_or(true)
            && let Some(list) = children.get_mut(root)
        {
            list.push(position);
        }
    }
    (children, root)
}
