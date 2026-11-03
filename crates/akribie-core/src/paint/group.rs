//! Grupo (caixa com faixa de título) e nota (canto dobrado).

use super::{Anchor, Fill, GroupDraw, NoteDraw, Stroke, TextRun, aria_of, dash_of};
use crate::layout::{PlacedGroup, PlacedNote};
use crate::sketch::{self, Ink};
use crate::text;
use crate::theme::Theme;

/// Respiro interno da nota.
const NOTE_PADDING: f32 = 8.0;

pub(super) fn paint_group(
    group: &PlacedGroup,
    theme: &Theme,
    fontsize: f32,
    ink: Ink,
) -> GroupDraw {
    let d = sketch::rect(group.rect, 0.0, ink);
    let label = TextRun {
        text: group.label.clone(),
        x: group.rect.x + text::NODE_PADDING,
        y: group.rect.y + fontsize * text::LINE_HEIGHT * 0.5,
        size: fontsize,
        color: theme.ink,
        anchor: Anchor::Start,
    };
    GroupDraw {
        id: group.id.clone(),
        aria_label: aria_of(std::slice::from_ref(&group.label)),
        stroke: Stroke {
            d,
            color: theme.hex(group.stroke).unwrap_or(theme.ink),
            width: group.width,
            dash: dash_of(group.style),
        },
        label: Some(label),
    }
}

pub(super) fn paint_note(note: &PlacedNote, theme: &Theme, fontsize: f32, ink: Ink) -> NoteDraw {
    let shape = sketch::note_shape(note.rect, ink);
    let color = theme.hex(note.fill).unwrap_or(theme.ink);
    let line_height = fontsize * text::LINE_HEIGHT;
    let text = note
        .text
        .iter()
        .enumerate()
        .map(|(index, line)| TextRun {
            text: line.clone(),
            x: note.rect.x + NOTE_PADDING,
            y: note.rect.y
                + NOTE_PADDING
                + line_height * (f32::from(u16::try_from(index).unwrap_or(0)) + 0.5),
            size: fontsize,
            color: theme.ink,
            anchor: Anchor::Start,
        })
        .collect();
    NoteDraw {
        id: format!("note-{}", note.span.line),
        aria_label: aria_of(&[note.text.join(" ")]),
        fill: Fill {
            d: shape.clone(),
            color,
            solid: true,
        },
        stroke: Stroke {
            d: shape,
            color: theme.ink,
            width: 1.5,
            dash: None,
        },
        text,
    }
}
