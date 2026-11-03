//! Caixa segura de texto: onde o rótulo pode viver sem encostar na forma.
//!
//! Cada forma tem um retângulo inscrito; descontado o respiro, sobra a área do texto. As mesmas
//! funções dimensionam o nó ([`node_box`]) e posicionam/embrulham o rótulo ([`text_box`]), o que
//! garante a igualdade entre o tamanho calculado e o texto desenhado: se o rótulo cabe no cálculo,
//! cabe no desenho.

use crate::geometry::Rect;
use crate::model::Shape;
use crate::sketch::cylinder_radius;
use crate::text::NODE_PADDING;

/// Fator do retângulo inscrito de uma elipse (`1/√2`).
const ELLIPSE: f32 = std::f32::consts::FRAC_1_SQRT_2;
/// Envelope interno da nuvem: os vales ficam em 80% do retângulo sobre a elipse.
const CLOUD: f32 = 0.58;

/// Retângulo inscrito (sem respiro) da forma.
fn inscribed(shape: Shape, rect: Rect) -> Rect {
    let (cx, cy) = rect.center();
    let centered = |w: f32, h: f32| Rect::new(cx - w * 0.5, cy - h * 0.5, w.max(1.0), h.max(1.0));
    match shape {
        Shape::Rect | Shape::Rounded | Shape::Note | Shape::Text => rect,
        Shape::Circle => centered(rect.w * ELLIPSE, rect.h * ELLIPSE),
        Shape::Diamond => centered(rect.w * 0.5, rect.h * 0.5),
        Shape::Hexagon => centered(rect.w * 0.75, rect.h * 0.5),
        Shape::Cloud => centered(rect.w * CLOUD, rect.h * CLOUD),
        Shape::Cylinder => {
            let ry = cylinder_radius(rect);
            Rect::new(
                rect.x,
                rect.y + 2.0 * ry,
                rect.w,
                (rect.h - 3.0 * ry).max(1.0),
            )
        }
    }
}

/// Área do rótulo: o inscrito, descontado o respiro.
pub fn text_box(shape: Shape, rect: Rect) -> Rect {
    let inner = inscribed(shape, rect);
    Rect::new(
        inner.x + NODE_PADDING,
        inner.y + NODE_PADDING,
        (inner.w - 2.0 * NODE_PADDING).max(1.0),
        (inner.h - 2.0 * NODE_PADDING).max(1.0),
    )
}

/// Menor caixa que contém o bloco de texto dentro do inscrito da forma.
pub(super) fn node_box(shape: Shape, text_w: f32, text_h: f32) -> (f32, f32) {
    let w = text_w + 2.0 * NODE_PADDING;
    let h = text_h + 2.0 * NODE_PADDING;
    match shape {
        Shape::Rect | Shape::Rounded | Shape::Note | Shape::Text => (w, h),
        Shape::Circle => (w / ELLIPSE, h / ELLIPSE),
        Shape::Diamond => (w / 0.5, h / 0.5),
        Shape::Hexagon => (w / 0.75, h / 0.5),
        Shape::Cloud => (w / CLOUD, h / CLOUD),
        Shape::Cylinder => {
            let mut height = h;
            for _ in 0..4 {
                height = h + 3.0 * cylinder_radius(Rect::new(0.0, 0.0, w, height));
            }
            (w, height)
        }
    }
}
