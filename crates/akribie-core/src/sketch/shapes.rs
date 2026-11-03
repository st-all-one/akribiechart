//! Geradores de `d` para as formas, em dois acabamentos: traço à mão ([`Ink::Rough`]) e traço
//! técnico ([`Ink::Exact`]).
//!
//! O acabamento técnico não é "o mesmo traço sem ruído": ele emite arcos reais (`A`) e segmentos
//! exatos, sem segunda passagem. É o que `sketch=off` desenha.

use super::Ink;
use super::MAX_OFFSET;
use super::helpers::{lineto, moveto, xy};
use super::rng::{jitter, passes};
use crate::geometry::{Point, Rect};
use crate::num::format as num;
use std::f32::consts::{FRAC_PI_2, PI};

/// Segmento de reta.
pub fn line(from: Point, to: Point, ink: Ink) -> String {
    passes(ink, |rng| {
        let a = jitter(rng, from, MAX_OFFSET * 0.5);
        let b = jitter(rng, to, MAX_OFFSET * 0.5);
        format!("{} {}", moveto(a), lineto(b))
    })
}

/// Fim do traço: volta ao primeiro ponto ou para no último.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trace {
    /// Traço fechado (formas).
    Closed,
    /// Traço aberto (rotas e dobras).
    Open,
}

/// Hachura a 45° ou a −45°.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tilt {
    /// Da esquerda para a direita.
    Forward,
    /// Espelhada.
    Mirrored,
}

/// Caminho aberto ou fechado por pontos.
pub fn polyline(points: &[Point], trace: Trace, ink: Ink) -> String {
    if points.len() < 2 {
        return String::new();
    }
    passes(ink, |rng| {
        let mut path = String::new();
        for (index, point) in points.iter().enumerate() {
            let point = jitter(rng, *point, MAX_OFFSET);
            if index == 0 {
                path.push_str(&moveto(point));
            } else {
                path.push(' ');
                path.push_str(&lineto(point));
            }
        }
        if trace == Trace::Closed {
            path.push_str(" Z");
        }
        path
    })
}

/// Cantos de um retângulo arredondado (4 amostras por canto).
fn rounded_corners(rect: Rect, radius: f32) -> Vec<Point> {
    let r = radius.min(rect.w * 0.5).min(rect.h * 0.5).max(0.0);
    if r <= 0.0 {
        return vec![
            (rect.x, rect.y),
            (rect.right(), rect.y),
            (rect.right(), rect.bottom()),
            (rect.x, rect.bottom()),
        ];
    }
    let mut points = Vec::with_capacity(16);
    let corners = [
        (rect.x + r, rect.y + r, PI),
        (rect.right() - r, rect.y + r, PI * 1.5),
        (rect.right() - r, rect.bottom() - r, 0.0),
        (rect.x + r, rect.bottom() - r, PI * 0.5),
    ];
    for (cx, cy, start) in corners {
        for step in 0..4_u8 {
            let fraction = f32::from(step) / 3.0;
            let angle = start + FRAC_PI_2 * fraction;
            points.push((cx + r * angle.cos(), cy + r * angle.sin()));
        }
    }
    points
}

/// Retângulo (com raio de canto).
pub fn rect(rect: Rect, radius: f32, ink: Ink) -> String {
    if ink == Ink::Exact {
        return exact_rect(rect, radius);
    }
    let points = rounded_corners(rect, radius);
    polyline(&points, Trace::Closed, ink)
}

/// Retângulo exato: segmentos e arcos reais, sem perturbação.
fn exact_rect(rect: Rect, radius: f32) -> String {
    let r = radius.min(rect.w * 0.5).min(rect.h * 0.5).max(0.0);
    let mut path = String::new();
    if r <= 0.0 {
        path.push_str(&moveto((rect.x, rect.y)));
        path.push(' ');
        path.push_str(&lineto((rect.right(), rect.y)));
        path.push(' ');
        path.push_str(&lineto((rect.right(), rect.bottom())));
        path.push(' ');
        path.push_str(&lineto((rect.x, rect.bottom())));
        path.push_str(" Z");
        return path;
    }
    let arc = |to: Point| format!("A{} {} 0 0 1 {} {}", num(r), num(r), num(to.0), num(to.1));
    path.push_str(&moveto((rect.x + r, rect.y)));
    path.push(' ');
    path.push_str(&lineto((rect.right() - r, rect.y)));
    path.push(' ');
    path.push_str(&arc((rect.right(), rect.y + r)));
    path.push(' ');
    path.push_str(&lineto((rect.right(), rect.bottom() - r)));
    path.push(' ');
    path.push_str(&arc((rect.right() - r, rect.bottom())));
    path.push(' ');
    path.push_str(&lineto((rect.x + r, rect.bottom())));
    path.push(' ');
    path.push_str(&arc((rect.x, rect.bottom() - r)));
    path.push(' ');
    path.push_str(&lineto((rect.x, rect.y + r)));
    path.push(' ');
    path.push_str(&arc((rect.x + r, rect.y)));
    path.push_str(" Z");
    path
}

/// Elipse.
pub fn ellipse(rect: Rect, ink: Ink) -> String {
    const KAPPA: f32 = 0.552_285;
    if ink == Ink::Exact {
        let (cx, cy) = rect.center();
        let rx = rect.w * 0.5;
        let ry = rect.h * 0.5;
        return format!(
            "{} A{} {} 0 1 0 {} {} A{} {} 0 1 0 {} {} Z",
            moveto((cx + rx, cy)),
            num(rx),
            num(ry),
            num(cx - rx),
            num(cy),
            num(rx),
            num(ry),
            num(cx + rx),
            num(cy)
        );
    }
    let (cx, cy) = rect.center();
    let rx = rect.w * 0.5;
    let ry = rect.h * 0.5;
    passes(ink, |rng| {
        let top = jitter(rng, (cx, cy - ry), MAX_OFFSET);
        let right = jitter(rng, (cx + rx, cy), MAX_OFFSET);
        let bottom = jitter(rng, (cx, cy + ry), MAX_OFFSET);
        let left = jitter(rng, (cx - rx, cy), MAX_OFFSET);
        let c1 = jitter(rng, (cx + rx * KAPPA, cy - ry), MAX_OFFSET);
        let c2 = jitter(rng, (cx + rx, cy - ry * KAPPA), MAX_OFFSET);
        let c3 = jitter(rng, (cx + rx, cy + ry * KAPPA), MAX_OFFSET);
        let c4 = jitter(rng, (cx + rx * KAPPA, cy + ry), MAX_OFFSET);
        let c5 = jitter(rng, (cx - rx * KAPPA, cy + ry), MAX_OFFSET);
        let c6 = jitter(rng, (cx - rx, cy + ry * KAPPA), MAX_OFFSET);
        let c7 = jitter(rng, (cx - rx, cy - ry * KAPPA), MAX_OFFSET);
        let c8 = jitter(rng, (cx - rx * KAPPA, cy - ry), MAX_OFFSET);
        format!(
            "{} C{} {} {} C{} {} {} C{} {} {} C{} {} {} Z",
            moveto(right),
            xy(c3),
            xy(c4),
            xy(bottom),
            xy(c5),
            xy(c6),
            xy(left),
            xy(c7),
            xy(c8),
            xy(top),
            xy(c1),
            xy(c2),
            xy(right)
        )
    })
}

/// Nota: retângulo com canto dobrado no topo direito, mais a dobra desenhada.
pub fn note_shape(rect: Rect, ink: Ink) -> String {
    const FOLD: f32 = 16.0;
    let fold = FOLD.min(rect.w * 0.25).min(rect.h * 0.25);
    let outline = [
        (rect.x, rect.y),
        (rect.right() - fold, rect.y),
        (rect.right(), rect.y + fold),
        (rect.right(), rect.bottom()),
        (rect.x, rect.bottom()),
    ];
    let crease = [
        (rect.right() - fold, rect.y),
        (rect.right() - fold, rect.y + fold),
        (rect.right(), rect.y + fold),
    ];
    format!(
        "{} {}",
        polyline(&outline, Trace::Closed, ink),
        polyline(&crease, Trace::Open, ink)
    )
}

/// Altura do arco da tampa do cilindro.
pub fn cylinder_radius(rect: Rect) -> f32 {
    (rect.h * 0.14).min(16.0)
}

/// Pontos de um arco elíptico, do ângulo `from` até `to` (radianos).
fn arc(cx: f32, cy: f32, rx: f32, ry: f32, from: f32, to: f32, samples: u8) -> Vec<Point> {
    let last = f32::from(samples);
    (1..=samples)
        .map(|index| {
            let angle = from + (to - from) * f32::from(index) / last;
            (cx + rx * angle.cos(), cy + ry * angle.sin())
        })
        .collect()
}

/// Cilindro: silhueta (lados + arco inferior + arco superior) e a borda frontal da tampa.
///
/// A tampa é desenhada **só pela metade da frente**; o arco superior da silhueta já é a metade de
/// trás. Antes as duas metades eram traçadas duas vezes e o traço à mão abria uma "aba" no topo.
pub fn cylinder(rect: Rect, ink: Ink) -> String {
    const SAMPLES: u8 = 10;
    let ry = cylinder_radius(rect);
    let (cx, _) = rect.center();
    let rx = rect.w * 0.5;
    let top = rect.y + ry;
    let bottom = rect.bottom() - ry;
    if ink == Ink::Exact {
        let body = format!(
            "{} {} A{} {} 0 0 0 {} {} {} A{} {} 0 0 0 {} {} Z",
            moveto((rect.x, top)),
            lineto((rect.x, bottom)),
            num(rx),
            num(ry),
            num(rect.right()),
            num(bottom),
            lineto((rect.right(), top)),
            num(rx),
            num(ry),
            num(rect.x),
            num(top)
        );
        let lid = format!(
            "{} A{} {} 0 0 1 {} {}",
            moveto((rect.x, top)),
            num(rx),
            num(ry),
            num(rect.right()),
            num(top)
        );
        return format!("{body} {lid}");
    }
    let mut body = vec![(rect.x, top), (rect.x, bottom)];
    body.extend(arc(cx, bottom, rx, ry, PI, 0.0, SAMPLES));
    body.push((rect.right(), top));
    body.extend(arc(cx, top, rx, ry, 0.0, -PI, SAMPLES));
    let lid = arc(cx, top, rx, ry, PI, 0.0, SAMPLES);
    format!(
        "{} {}",
        polyline(&body, Trace::Closed, ink),
        polyline(&lid, Trace::Open, ink)
    )
}
