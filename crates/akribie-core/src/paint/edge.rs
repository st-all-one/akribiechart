//! Aresta: rota no acabamento do documento, setas e rótulo com fundo.
//!
//! O rótulo nasce do **meio por comprimento de arco** da rota (não do índice central), deslocado
//! pela normal do trecho: assim ele fica sempre centrado no traço, nunca sobre o nó de destino.

use super::{Anchor, EdgeDraw, Fill, LABEL_PAD, LabelDraw, Stroke, TextRun, dash_of};
use crate::geometry::{self, Point, Rect};
use crate::layout::PlacedEdge;
use crate::sketch::{self, Ink, Trace};
use crate::text::{self, Gauge};
use crate::theme::Theme;

pub(super) fn paint_edge(
    edge: &PlacedEdge,
    theme: &Theme,
    fontsize: f32,
    gauge: &Gauge,
    ink: Ink,
) -> EdgeDraw {
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
    let height = fontsize * text::LINE_HEIGHT;
    let label = edge.label.as_ref().and_then(|text| {
        let (middle, normal) = label_anchor(points)?;
        let width = gauge.advance_px(text, fontsize);
        let rect = label_rect(points, middle, normal, width, height);
        Some(LabelDraw {
            text: TextRun {
                text: text.clone(),
                x: rect.center().0,
                y: rect.center().1,
                size: fontsize,
                color: theme.ink,
                anchor: Anchor::Middle,
            },
            rect,
        })
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

/// Ponto médio da rota e a normal que aponta para o lado "de cima" do traço.
fn label_anchor(route: &[Point]) -> Option<(Point, Point)> {
    let (middle, direction) = geometry::route_midpoint(route)?;
    let (dx, dy) = direction;
    let mut normal = (-dy, dx);
    if normal.1 > 0.0 || (normal.1 == 0.0 && normal.0 < 0.0) {
        normal = (-normal.0, -normal.1);
    }
    Some((middle, normal))
}

/// Retângulo do rótulo: parte da folga mínima e se afasta da curva até ela valer de fato.
///
/// Numa rota curva, o deslocamento perpendicular medido no meio não basta: os cantos do retângulo
/// alcançam o traço. O laço aumenta o afastamento em passos fixos (determinístico) até a distância
/// mínima entre rota e retângulo ser, ao menos, [`LABEL_PAD`].
fn label_rect(route: &[Point], middle: Point, normal: Point, width: f32, height: f32) -> Rect {
    let mut shift = height * 0.5 + LABEL_PAD;
    let mut rect = rect_at(middle, normal, width, height, shift);
    while shift < 128.0 && clearance(route, rect) + 0.001 < LABEL_PAD {
        shift += 1.0;
        rect = rect_at(middle, normal, width, height, shift);
    }
    rect
}

fn rect_at(middle: Point, normal: Point, width: f32, height: f32, shift: f32) -> Rect {
    let center = (middle.0 + normal.0 * shift, middle.1 + normal.1 * shift);
    Rect::new(
        center.0 - width * 0.5 - LABEL_PAD,
        center.1 - height * 0.5,
        width + 2.0 * LABEL_PAD,
        height,
    )
}

/// Menor distância entre a rota (amostrada por segmento) e o retângulo.
fn clearance(route: &[Point], rect: Rect) -> f32 {
    const SAMPLES: u8 = 16;
    let mut minimum = f32::INFINITY;
    for pair in route.windows(2) {
        let Some((from, rest)) = pair.split_first() else {
            continue;
        };
        let Some(to) = rest.first() else {
            continue;
        };
        let (from, to) = (*from, *to);
        for step in 0..=SAMPLES {
            let t = f32::from(step) / f32::from(SAMPLES);
            let point = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
            minimum = minimum.min(distance_to_rect(point, rect));
        }
    }
    minimum
}

fn distance_to_rect(point: Point, rect: Rect) -> f32 {
    let dx = (rect.x - point.0).max(point.0 - rect.right()).max(0.0);
    let dy = (rect.y - point.1).max(point.1 - rect.bottom()).max(0.0);
    dx.hypot(dy)
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
