//! Formatação de `d` (números, comandos) e recorte no retângulo.

use crate::geometry::{Point, Rect};
use crate::num::format as num;

pub(super) fn moveto(point: Point) -> String {
    format!("M{} {}", num(point.0), num(point.1))
}

pub(super) fn lineto(point: Point) -> String {
    format!("L{} {}", num(point.0), num(point.1))
}

pub(super) fn xy(point: Point) -> String {
    format!("{} {}", num(point.0), num(point.1))
}

pub(super) fn clamp_to_rect(point: Point, rect: Rect) -> Point {
    (
        point.0.clamp(rect.x, rect.right()),
        point.1.clamp(rect.y, rect.bottom()),
    )
}
