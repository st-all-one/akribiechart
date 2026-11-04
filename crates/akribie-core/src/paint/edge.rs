//! Aresta: rota no acabamento do documento, setas e rótulo com fundo.
//!
//! A posição do rótulo já vem decidida do layout (`PlacedEdge::label_rect`, com colisão
//! resolvida e a rota reservada no dagre): aqui só se transforma geometria em desenho.

use super::{Anchor, EdgeDraw, Fill, LabelDraw, Stroke, TextRun, dash_of};
use crate::geometry::{Point, Rect};
use crate::layout::PlacedEdge;
use crate::sketch::{self, Ink, Trace};
use crate::text;
use crate::theme::Theme;

pub(super) fn paint_edge(edge: &PlacedEdge, theme: &Theme, fontsize: f32, ink: Ink) -> EdgeDraw {
    let route = sketch::polyline(&edge.route, Trace::Open, ink);
    let color = theme.hex(edge.stroke).unwrap_or(theme.ink);
    let points = &edge.route;
    let head = if edge.head {
        arrow_at_end(points, color)
    } else {
        None
    };
    let tail = if edge.tail {
        arrow_at_start(points, color)
    } else {
        None
    };
    let label = edge.label_rect.map(|rect| LabelDraw {
        text: label_runs(&edge.label_lines, rect, fontsize, theme.ink),
        rect,
    });
    EdgeDraw {
        id: format!("{}->{}#{}", edge.from, edge.to, edge.ordinal),
        aria_label: edge.label.clone(),
        route: Stroke {
            d: route,
            color,
            width: edge.width,
            dash: dash_of(edge.style),
        },
        head,
        tail,
        label,
    }
}

/// Uma linha de texto por linha do rótulo, centradas na caixa reservada.
fn label_runs(lines: &[String], rect: Rect, fontsize: f32, color: &'static str) -> Vec<TextRun> {
    let line_height = fontsize * text::LINE_HEIGHT;
    let center = rect.center().0;
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| TextRun {
            text: line.clone(),
            x: center,
            y: rect.y + line_height * (f32::from(u16::try_from(index).unwrap_or(0)) + 0.5),
            size: fontsize,
            color,
            anchor: Anchor::Middle,
        })
        .collect()
}

/// Seta no fim da rota (lado `to`).
fn arrow_at_end(points: &[Point], color: &'static str) -> Option<Fill> {
    let tip = points.last().copied()?;
    let back = points.get(points.len().saturating_sub(2)).copied()?;
    Some(Fill {
        d: sketch::arrow_head(tip, back),
        color,
        solid: true,
    })
}

/// Seta no começo da rota (lado `from`, seta bidirecional).
fn arrow_at_start(points: &[Point], color: &'static str) -> Option<Fill> {
    let tip = points.first().copied()?;
    let back = points.get(1).copied()?;
    Some(Fill {
        d: sketch::arrow_head(tip, back),
        color,
        solid: true,
    })
}
