//! Nuvem: contorno serrilhado desenhado dentro do retângulo do nó.
//!
//! Os oito raios alternados e os lóbulos de `bumpy_closed` mantêm o traço dentro da caixa que o
//! layout reservou, em qualquer acabamento.

use super::helpers::{moveto, xy};
use super::shapes::polyline;
use super::{Ink, Trace};
use crate::geometry::{Point, Rect};
use std::f32::consts::TAU;
use std::fmt::Write as _;

/// Pontos da nuvem: oito raios alternados, todos dentro do retângulo do nó.
fn cloud_points(rect: Rect) -> Vec<Point> {
    let (cx, cy) = rect.center();
    let rx = rect.w * 0.5;
    let ry = rect.h * 0.5;
    (0..20_u8)
        .map(|step| {
            let angle = TAU * f32::from(step) / 20.0;
            let wobble = if step % 2 == 0 { 1.0 } else { 0.9 };
            (
                cx + rx * wobble * angle.cos(),
                cy + ry * wobble * angle.sin(),
            )
        })
        .collect()
}

/// Contorno fechado com um lóbulo por aresta: cada trecho é uma cúbica cujos controles se
/// afastam do centro, o que dá o serrilhado de nuvem sem cruzar o próprio traço.
fn bumpy_closed(points: &[Point], bulge: f32) -> String {
    let count = points.len();
    if count < 3 {
        return polyline(points, Trace::Closed, Ink::Exact);
    }
    let (cx, cy) = points.iter().fold((0.0_f32, 0.0_f32), |(ax, ay), point| {
        (ax + point.0, ay + point.1)
    });
    let divisor = f32::from(u16::try_from(count).unwrap_or(u16::MAX));
    let center = (cx / divisor, cy / divisor);
    let Some(first) = points.first().copied() else {
        return String::new();
    };
    let mut path = moveto(first);
    for index in 0..count {
        let next = index.saturating_add(1).checked_rem(count);
        let (Some(from), Some(to)) = (
            points.get(index).copied(),
            next.and_then(|next| points.get(next)).copied(),
        ) else {
            continue;
        };
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        let length = dx.hypot(dy);
        if length <= f32::EPSILON {
            continue;
        }
        let mut normal = (dy / length, -dx / length);
        let midpoint = (f32::midpoint(from.0, to.0), f32::midpoint(from.1, to.1));
        if (normal.0 * (midpoint.0 - center.0) + normal.1 * (midpoint.1 - center.1)) < 0.0 {
            normal = (-normal.0, -normal.1);
        }
        let push = |point: Point| (point.0 + normal.0 * bulge, point.1 + normal.1 * bulge);
        path.write_fmt(format_args!(
            " C{} {} {}",
            xy(push(from)),
            xy(push(to)),
            xy(to)
        ))
        .unwrap_or(());
    }
    path.push_str(" Z");
    path
}

/// Nuvem: oito lóbulos sobre a elipse inscrita, dentro do retângulo do nó.
pub fn cloud(rect: Rect, ink: Ink) -> String {
    let points = cloud_points(rect);
    if ink == Ink::Exact {
        return bumpy_closed(&points, rect.h.min(rect.w) * 0.05);
    }
    polyline(&points, Trace::Closed, ink)
}
