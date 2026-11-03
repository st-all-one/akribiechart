//! Hachura e zigue-zague (preenchimento recortado pelo `clipPath`).

use super::helpers::{clamp_to_rect, lineto, moveto};
use super::shapes::Tilt;
use crate::geometry::Rect;

/// Hachura a 45° (ou a −45°) cobrindo o retângulo; o `clipPath` corta na forma.
pub fn hatch(rect: Rect, step: f32, tilt: Tilt) -> String {
    if step <= 0.5 {
        return String::new();
    }
    let mut segments: Vec<String> = Vec::new();
    let mut offset = -rect.h;
    while offset < rect.w + rect.h {
        let from = if tilt == Tilt::Mirrored {
            (rect.x + offset, rect.bottom())
        } else {
            (rect.x + offset, rect.y)
        };
        let to = if tilt == Tilt::Mirrored {
            (rect.x + offset + rect.h, rect.y)
        } else {
            (rect.x + offset + rect.h, rect.bottom())
        };
        segments.push(format!(
            "{} {}",
            moveto(clamp_to_rect(from, rect)),
            lineto(clamp_to_rect(to, rect))
        ));
        offset += step;
    }
    segments.join(" ")
}

/// Zigue-zague horizontal cobrindo o retângulo.
pub fn zigzag(rect: Rect, amplitude: f32, step: f32) -> String {
    if step <= 0.5 {
        return String::new();
    }
    let mut segments: Vec<String> = Vec::new();
    let mut y = rect.y + step * 0.5;
    while y < rect.bottom() {
        segments.push(format!(
            "{} {} {}",
            moveto((rect.x, y - amplitude)),
            lineto((rect.x + rect.w * 0.5, y + amplitude)),
            lineto((rect.right(), y - amplitude))
        ));
        y += step;
    }
    segments.join(" ")
}
