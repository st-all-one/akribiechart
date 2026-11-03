//! `layout=free`: posições declaradas com `at` (canto superior esquerdo) e `size`.

use crate::geometry::Rect;
use crate::model::Chart;

pub(super) fn run(chart: &Chart, sizes: &[(f32, f32)]) -> Vec<Rect> {
    chart
        .nodes
        .iter()
        .zip(sizes)
        .map(|(node, (width, height))| {
            let (x, y) = node.at.unwrap_or((0.0, 0.0));
            Rect::new(x, y, *width, *height)
        })
        .collect()
}
