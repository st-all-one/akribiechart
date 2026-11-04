//! `layout=tree`: Reingold–Tilford simplificado — folhas em ordem, pais centrados.
//!
//! Raízes são os nós sem aresta de entrada, na ordem de declaração; filhos seguem a ordem das
//! arestas. Nós não alcançados (ciclo puro) viram raízes extras ao final, para nada desaparecer.

use indexmap::IndexMap;

use crate::geometry::Rect;
use crate::model::{Chart, Direction};

/// Folga entre irmãos.
const H_GAP: f32 = 40.0;
/// Folga entre níveis.
const V_GAP: f32 = 80.0;

struct Tree<'a> {
    children: &'a [Vec<usize>],
    sizes: &'a [(f32, f32)],
    direction: Direction,
    horizontal: bool,
    center: Vec<f32>,
    depth: Vec<usize>,
    cursor: f32,
}

pub(super) fn run(chart: &Chart, sizes: &[(f32, f32)]) -> Vec<Rect> {
    let count = sizes.len();
    if count == 0 {
        return Vec::new();
    }
    let (children, mut roots) = forest(chart, count);
    let horizontal = matches!(chart.direction, Direction::Lr | Direction::Rl);
    let mut tree = Tree {
        children: &children,
        sizes,
        direction: chart.direction,
        horizontal,
        center: vec![0.0; count],
        depth: vec![0; count],
        cursor: 0.0,
    };
    let mut placed = vec![false; count];
    for root in roots.clone() {
        tree.place(root, 0, &mut placed);
    }
    for index in 0..count {
        if !placed.get(index).copied().unwrap_or(true) {
            roots.push(index);
            tree.place(index, 0, &mut placed);
        }
    }
    tree.rects(count)
}

impl Tree<'_> {
    fn place(&mut self, index: usize, level: usize, placed: &mut [bool]) {
        if placed.get(index).copied().unwrap_or(true) {
            return;
        }
        if let Some(slot) = placed.get_mut(index) {
            *slot = true;
        }
        if let Some(slot) = self.depth.get_mut(index) {
            *slot = level;
        }
        let children = self.children.get(index).cloned().unwrap_or_default();
        if children.is_empty() {
            let total = self.main_extent(index);
            if let Some(slot) = self.center.get_mut(index) {
                *slot = self.cursor + total * 0.5;
            }
            self.cursor += total + H_GAP;
            return;
        }
        let mut sum = 0.0_f32;
        let mut count = 0.0_f32;
        for child in children {
            self.place(child, level.saturating_add(1), placed);
            if let Some(center) = self.center.get(child) {
                sum += *center;
                count += 1.0;
            }
        }
        if let Some(slot) = self.center.get_mut(index) {
            *slot = if count > 0.0 {
                sum / count
            } else {
                self.cursor
            };
        }
    }

    /// Extensão do nó no eixo principal (onde os irmãos se distribuem).
    fn main_extent(&self, index: usize) -> f32 {
        let (width, height) = self.sizes.get(index).copied().unwrap_or((0.0, 0.0));
        if self.horizontal { height } else { width }
    }

    fn cross_extent(&self, index: usize) -> f32 {
        let (width, height) = self.sizes.get(index).copied().unwrap_or((0.0, 0.0));
        if self.horizontal { width } else { height }
    }

    fn rects(&self, count: usize) -> Vec<Rect> {
        let max_depth = self.depth.iter().copied().max().unwrap_or(0);
        let mut row_extent = vec![0.0_f32; max_depth.saturating_add(1)];
        for index in 0..count {
            if let Some(depth) = self.depth.get(index)
                && let Some(slot) = row_extent.get_mut(*depth)
            {
                *slot = slot.max(self.cross_extent(index));
            }
        }
        let mut row_offset = vec![0.0_f32; max_depth.saturating_add(1)];
        for level in 1..=max_depth {
            let above = level.saturating_sub(1);
            let previous = row_offset.get(above).copied().unwrap_or(0.0)
                + row_extent.get(above).copied().unwrap_or(0.0);
            if let Some(slot) = row_offset.get_mut(level) {
                *slot = previous + V_GAP;
            }
        }
        let total_cross = row_offset.get(max_depth).copied().unwrap_or(0.0)
            + row_extent.get(max_depth).copied().unwrap_or(0.0);
        (0..count)
            .map(|index| {
                let (width, height) = self.sizes.get(index).copied().unwrap_or((0.0, 0.0));
                let level = self.depth.get(index).copied().unwrap_or(0);
                let cross = row_offset.get(level).copied().unwrap_or(0.0)
                    + row_extent.get(level).copied().unwrap_or(0.0) * 0.5;
                let main = self.center.get(index).copied().unwrap_or(0.0);
                let (x, y) = if self.horizontal {
                    (cross - width * 0.5, main - height * 0.5)
                } else {
                    (main - width * 0.5, cross - height * 0.5)
                };
                let (x, y) = match self.direction {
                    Direction::Bt => (x, total_cross - (y + height)),
                    Direction::Rl => (total_cross - (x + width), y),
                    Direction::Lr | Direction::Tb => (x, y),
                };
                Rect::new(x, y, width, height)
            })
            .collect()
    }
}

/// Floresta de arestas sem ciclo: cada nó tem no máximo um pai.
fn forest(chart: &Chart, count: usize) -> (Vec<Vec<usize>>, Vec<usize>) {
    let index: IndexMap<&str, usize> = chart
        .nodes
        .iter()
        .enumerate()
        .map(|(position, node)| (node.id.as_str(), position))
        .collect();
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut has_parent = vec![false; count];
    for edge in &chart.edges {
        let (Some(&from), Some(&to)) = (index.get(edge.from.as_str()), index.get(edge.to.as_str()))
        else {
            continue;
        };
        if from == to || has_parent.get(to).copied().unwrap_or(true) {
            continue;
        }
        if children.get(from).is_some_and(|list| list.contains(&to)) {
            continue;
        }
        if let Some(list) = children.get_mut(from) {
            list.push(to);
        }
        if let Some(flag) = has_parent.get_mut(to) {
            *flag = true;
        }
    }
    let roots = (0..count)
        .filter(|position| !has_parent.get(*position).copied().unwrap_or(false))
        .collect();
    (children, roots)
}
