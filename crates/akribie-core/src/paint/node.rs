//! Nó: forma com o acabamento do documento, preenchimento e rótulo na área segura.

use super::{Anchor, CORNER_RADIUS, Fill, NodeDraw, Stroke, aria_of, dash_of, text_runs};
use crate::geometry::{Point, Rect};
use crate::layout::{PlacedNode, text_box};
use crate::model::{FillStyle, Shape};
use crate::sketch::{self, Ink, Tilt, Trace};
use crate::theme::{Theme, Token};

pub(super) fn paint_node(node: &PlacedNode, theme: &Theme, fontsize: f32, ink: Ink) -> NodeDraw {
    let shape = shape_path(node, ink);
    let clip =
        if node.fill != Token::None && node.fillstyle != FillStyle::Solid && !shape.is_empty() {
            Some(shape.clone())
        } else {
            None
        };
    let fill = fill_path(node, &shape, theme);
    let stroke = if node.shape == Shape::Text || node.stroke == Token::None {
        None
    } else {
        theme.hex(node.stroke).map(|color| Stroke {
            d: shape,
            color,
            width: node.width,
            dash: dash_of(node.style),
        })
    };
    NodeDraw {
        id: node.id.clone(),
        aria_label: aria_of(&[node.label.join(" ")]),
        clip,
        fill,
        stroke,
        text: text_runs(
            &node.label,
            text_box(node.shape, node.rect),
            fontsize,
            theme.ink,
            Anchor::Middle,
        ),
    }
}

fn fill_path(node: &PlacedNode, shape: &str, theme: &Theme) -> Option<Fill> {
    let color = theme.hex(node.fill)?;
    if node.fillstyle == FillStyle::Solid {
        return Some(Fill {
            d: shape.to_string(),
            color,
            solid: true,
        });
    }
    let step = 8.0 + node.width * 2.0;
    let d = match node.fillstyle {
        FillStyle::Hachure => sketch::hatch(node.rect, step, Tilt::Forward),
        FillStyle::CrossHatch => format!(
            "{} {}",
            sketch::hatch(node.rect, step, Tilt::Forward),
            sketch::hatch(node.rect, step, Tilt::Mirrored)
        ),
        FillStyle::Zigzag => sketch::zigzag(node.rect, 4.0, step),
        FillStyle::Solid => shape.to_string(),
    };
    Some(Fill {
        d,
        color,
        solid: false,
    })
}

/// `d` da forma, no acabamento do documento e com semente do id.
fn shape_path(node: &PlacedNode, ink: Ink) -> String {
    match node.shape {
        Shape::Rect => sketch::rect(node.rect, 0.0, ink),
        Shape::Rounded => sketch::rect(node.rect, CORNER_RADIUS, ink),
        Shape::Circle => sketch::ellipse(node.rect, ink),
        Shape::Diamond => sketch::polyline(&diamond(node.rect), Trace::Closed, ink),
        Shape::Hexagon => sketch::polyline(&hexagon(node.rect), Trace::Closed, ink),
        Shape::Cloud => sketch::cloud(node.rect, ink),
        Shape::Cylinder => sketch::cylinder(node.rect, ink),
        Shape::Note => sketch::note_shape(node.rect, ink),
        Shape::Text => String::new(),
    }
}

fn diamond(rect: Rect) -> Vec<Point> {
    let (cx, cy) = rect.center();
    vec![
        (cx, rect.y),
        (rect.right(), cy),
        (cx, rect.bottom()),
        (rect.x, cy),
    ]
}

fn hexagon(rect: Rect) -> Vec<Point> {
    let (_, cy) = rect.center();
    let quarter = rect.w * 0.25;
    vec![
        (rect.x + quarter, rect.y),
        (rect.right() - quarter, rect.y),
        (rect.right(), cy),
        (rect.right() - quarter, rect.bottom()),
        (rect.x + quarter, rect.bottom()),
        (rect.x, cy),
    ]
}
