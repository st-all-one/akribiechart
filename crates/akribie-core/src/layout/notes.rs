//! Notas: `at` manda; sem ele, a nota fica na diagonal inferior direita do nó de `near=`
//! (abaixo do rank e à direita da coluna, sem disputar com os dois).

use indexmap::IndexMap;

use crate::geometry::Rect;
use crate::model::Chart;
use crate::text::{self, Gauge};

use super::{PlacedNode, PlacedNote};

/// Distância entre um nó e a nota ancorada nele.
const NOTE_GAP: f32 = 24.0;
/// Respiro interno de uma nota.
const NOTE_PADDING: f32 = 8.0;

pub(super) fn note_boxes(
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
