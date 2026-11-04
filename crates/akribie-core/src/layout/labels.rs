//! Rótulos: medição de grupo e posição de rótulo de aresta.
//!
//! Duas decisões vivem aqui. A primeira é a faixa de título de um grupo: medida e quebrada na
//! largura máxima de rótulo, para o texto alargar a caixa em vez de vazar por ela. A segunda é a
//! posição do rótulo de uma aresta, procurada ao longo do próprio traço em candidatos
//! determinísticos até não encostar em nó, faixa de grupo, outro rótulo nem no traço. Nada aqui
//! depende de cor ou de documento: só geometria.

use crate::geometry::{self, Point, Rect};
use crate::model::Chart;
use crate::paint::LABEL_PAD;
use crate::text::{self, Gauge};

use super::{PlacedEdge, PlacedGroup, PlacedNode};

/// Frações do comprimento de arco onde o rótulo pode ser ancorado, do meio para as pontas.
const FRACTIONS: [f32; 9] = [0.5, 0.42, 0.58, 0.34, 0.66, 0.26, 0.74, 0.18, 0.82];
/// Passo do afastamento perpendicular, em pixels.
const SHIFT_STEP: f32 = 6.0;
/// Afastamentos testados por âncora (nove passos = 48 px além da folga base).
const SHIFT_STEPS: u8 = 9;
/// Teto do afastamento no último recurso, quando nenhum candidato serve.
const MAX_SHIFT: f32 = 128.0;

/// Altura total de `lines` linhas de rótulo no tamanho de fonte dado.
pub(super) fn text_height(lines: usize, fontsize: f32) -> f32 {
    let count = u16::try_from(lines).unwrap_or(u16::MAX);
    f32::from(count) * fontsize * text::LINE_HEIGHT
}

/// Faixa de título de um grupo já traduzida em retângulo, para servir de obstáculo.
fn title_band(group: &PlacedGroup, fontsize: f32) -> Rect {
    Rect::new(
        group.rect.x,
        group.rect.y,
        group.rect.w,
        text_height(group.label_lines.len(), fontsize),
    )
}

/// Rótulo de grupo quebrado na largura máxima, com a largura do texto e a altura da faixa.
pub(super) fn group_metrics(gauge: &Gauge, label: &str, fontsize: f32) -> (Vec<String>, f32, f32) {
    let lines = gauge.wrap(label, fontsize, text::MAX_LABEL_W);
    let text_width = max_width(&lines, gauge, fontsize);
    let band = text_height(lines.len(), fontsize);
    (lines, text_width + 2.0 * text::NODE_PADDING, band)
}

/// Caixa reservada a cada rótulo de aresta, na ordem de `chart.edges` (`0,0` sem rótulo).
pub(super) fn edge_sizes(chart: &Chart, gauge: &Gauge) -> Vec<(f32, f32)> {
    chart
        .edges
        .iter()
        .map(|edge| match edge.label.as_deref() {
            Some(label) => {
                let lines = gauge.wrap(label, chart.fontsize, text::MAX_LABEL_W);
                let width = max_width(&lines, gauge, chart.fontsize) + 2.0 * LABEL_PAD;
                (width, text_height(lines.len(), chart.fontsize))
            }
            None => (0.0, 0.0),
        })
        .collect()
}

fn max_width(lines: &[String], gauge: &Gauge, fontsize: f32) -> f32 {
    lines
        .iter()
        .map(|line| gauge.advance_px(line, fontsize))
        .fold(0.0_f32, f32::max)
}

/// Posiciona os rótulos de aresta, na ordem de `chart.edges`, sem sobreposição.
pub(super) fn place(
    chart: &Chart,
    gauge: &Gauge,
    nodes: &[PlacedNode],
    groups: &[PlacedGroup],
    edges: &mut [PlacedEdge],
) {
    let mut blocked: Vec<Rect> = nodes.iter().map(|node| node.rect).collect();
    blocked.extend(groups.iter().map(|group| title_band(group, chart.fontsize)));
    for edge in &mut *edges {
        edge.label_rect = None;
        edge.label_lines.clear();
        let Some(label) = edge.label.as_deref() else {
            continue;
        };
        let lines = gauge.wrap(label, chart.fontsize, text::MAX_LABEL_W);
        let width = max_width(&lines, gauge, chart.fontsize);
        let height = text_height(lines.len(), chart.fontsize);
        let rect = choose(&edge.route, width, height, &blocked);
        edge.label_rect = Some(rect);
        edge.label_lines = lines;
        blocked.push(rect);
    }
}

/// Primeiro candidato que não toca nada; sem candidato livre, o afastamento mínimo na âncora média.
fn choose(route: &[Point], width: f32, height: f32, blocked: &[Rect]) -> Rect {
    let base = height * 0.5 + LABEL_PAD;
    for fraction in FRACTIONS {
        let Some((middle, direction)) = geometry::route_point_at(route, fraction) else {
            continue;
        };
        let normal = normal_of(direction);
        for step in 0..SHIFT_STEPS {
            let shift = base + f32::from(step) * SHIFT_STEP;
            let rect = rect_at(middle, normal, width, height, shift);
            if fits(route, rect, blocked) {
                return rect;
            }
        }
    }
    fallback(route, width, height)
}

/// Normal do traço que aponta para o lado "de cima" (é o lado onde o rótulo nasce).
fn normal_of(direction: Point) -> Point {
    let normal = (-direction.1, direction.0);
    if normal.1 > 0.0 || (normal.1 == 0.0 && normal.0 < 0.0) {
        (-normal.0, -normal.1)
    } else {
        normal
    }
}

fn fits(route: &[Point], rect: Rect, blocked: &[Rect]) -> bool {
    clearance(route, rect) + 0.001 >= LABEL_PAD
        && !blocked.iter().any(|other| rect.intersects(*other))
}

fn fallback(route: &[Point], width: f32, height: f32) -> Rect {
    let Some((middle, direction)) = geometry::route_midpoint(route) else {
        return Rect::new(0.0, 0.0, width + 2.0 * LABEL_PAD, height);
    };
    let normal = normal_of(direction);
    let mut shift = height * 0.5 + LABEL_PAD;
    let mut rect = rect_at(middle, normal, width, height, shift);
    while shift < MAX_SHIFT && clearance(route, rect) + 0.001 < LABEL_PAD {
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
