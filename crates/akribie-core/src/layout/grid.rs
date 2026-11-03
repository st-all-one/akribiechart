//! `layout=grid` e `layout=stack`: fileiras regulares, sem arestas como restrição.

use crate::geometry::Rect;
use crate::model::Layout;

/// Folga entre células da grade.
const GAP_GRID: f32 = 40.0;
/// Folga entre itens da pilha.
const GAP_STACK: f32 = 28.0;

pub(super) fn run(sizes: &[(f32, f32)], layout: Layout) -> Vec<Rect> {
    if layout == Layout::Stack {
        return stack(sizes);
    }
    let count = sizes.len();
    if count == 0 {
        return Vec::new();
    }
    let mut columns = 1;
    while columns * columns < count {
        columns += 1;
    }
    let rows = count.div_ceil(columns);
    let mut column_width = vec![0.0_f32; columns];
    let mut row_height = vec![0.0_f32; rows];
    for (index, (width, height)) in sizes.iter().enumerate() {
        let column = index % columns;
        let row = index / columns;
        if let Some(slot) = column_width.get_mut(column) {
            *slot = slot.max(*width);
        }
        if let Some(slot) = row_height.get_mut(row) {
            *slot = slot.max(*height);
        }
    }
    let mut offset_x = vec![0.0_f32; columns];
    for column in 1..columns {
        let previous = offset_x.get(column - 1).copied().unwrap_or(0.0)
            + column_width.get(column - 1).copied().unwrap_or(0.0);
        if let Some(slot) = offset_x.get_mut(column) {
            *slot = previous + GAP_GRID;
        }
    }
    let mut offset_y = vec![0.0_f32; rows];
    for row in 1..rows {
        let previous = offset_y.get(row - 1).copied().unwrap_or(0.0)
            + row_height.get(row - 1).copied().unwrap_or(0.0);
        if let Some(slot) = offset_y.get_mut(row) {
            *slot = previous + GAP_GRID;
        }
    }
    sizes
        .iter()
        .enumerate()
        .map(|(index, (width, height))| {
            let x = offset_x.get(index % columns).copied().unwrap_or(0.0);
            let y = offset_y.get(index / columns).copied().unwrap_or(0.0);
            Rect::new(x, y, *width, *height)
        })
        .collect()
}

/// Uma coluna, de cima para baixo (usado por `layout=stack` e como fallback do dagre).
pub(super) fn stack(sizes: &[(f32, f32)]) -> Vec<Rect> {
    let mut y = 0.0_f32;
    sizes
        .iter()
        .map(|(width, height)| {
            let rect = Rect::new(0.0, y, *width, *height);
            y += height + GAP_STACK;
            rect
        })
        .collect()
}
