//! Traço à mão: RNG determinístico e geradores de `d` para SVG.
//!
//! O desenho é "feito à mão" mas **byte-estável**: a semente vem do id (`FNV-1a`) e todo gerador
//! perturba os pontos uma única vez por passagem. Duas passagens por traço dão o traço duplo
//! característico sem depender de relógio, `thread_rng` ou ordem de iteração.

use crate::geometry::Point;
use helpers::{lineto, moveto};

mod cloud;
mod hatch;
mod helpers;
mod rng;
mod shapes;

pub use cloud::cloud;
pub use hatch::{hatch, zigzag};
pub use rng::{Rng, seed_for};
pub use shapes::{
    Tilt, Trace, cylinder, cylinder_radius, ellipse, line, note_shape, polyline, rect,
};

/// Deslocamento máximo de um ponto, em pixels.
pub const MAX_OFFSET: f32 = 2.0;

/// Acabamento do traço.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    /// Traço à mão, com semente estável por id.
    Rough(u32),
    /// Traço técnico: geometria exata, uma passagem, sem perturbação.
    Exact,
}

/// Cabeça de seta preenchida, apontando de `back` para `tip`.
pub fn arrow_head(tip: Point, back: Point) -> String {
    const LEN: f32 = 12.0;
    const WIDE: f32 = 8.0;
    let dx = tip.0 - back.0;
    let dy = tip.1 - back.1;
    let length = (dx * dx + dy * dy).sqrt();
    if length < f32::EPSILON {
        return String::new();
    }
    let ux = dx / length;
    let uy = dy / length;
    let base = (tip.0 - ux * LEN, tip.1 - uy * LEN);
    let left = (base.0 - uy * WIDE, base.1 + ux * WIDE);
    let right = (base.0 + uy * WIDE, base.1 - ux * WIDE);
    format!("{} {} {} Z", moveto(tip), lineto(left), lineto(right))
}

#[cfg(test)]
mod tests;
