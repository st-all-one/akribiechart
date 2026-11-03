//! Peças comuns do export: ids, `index`, campos base e rótulos.

use serde_json::{Value, json};

use crate::geometry::{Point, Rect};
use crate::layout::Placed;
use crate::model::FontFamily;
use crate::model::{Shape, StrokeStyle};
use crate::num;
use crate::sketch::seed_for;
use crate::theme::{Theme, Token};

/// Alfabeto do `index` (ordem ASCII = ordem do alfabeto).
const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Insere uma chave sem indexar (`indexing_slicing` é negado).
pub(super) fn set(element: &mut Value, key: &str, value: Value) {
    if let Some(map) = element.as_object_mut() {
        map.insert(key.to_string(), value);
    }
}

/// Marca a origem akribie no `customData` (o import devolve grupo/nota fielmente).
pub(super) fn tag(element: &mut Value, kind: &str) {
    set(element, "customData", json!({ "akr": kind }));
}

pub(super) fn index_key(position: usize) -> String {
    let mut digits = [b'0'; 4];
    let mut rest = position;
    for slot in digits.iter_mut().rev() {
        *slot = BASE62.get(rest % BASE62.len()).copied().unwrap_or(b'0');
        rest /= BASE62.len();
    }
    format!("a{}V", String::from_utf8_lossy(&digits))
}

pub(super) fn element_id(id: &str) -> String {
    format!("akr-{id}")
}

/// Campos comuns a todo elemento.
pub(super) fn base(
    id: &str,
    kind: &str,
    rect: Rect,
    stroke: &'static str,
    background: &'static str,
    seed: u32,
) -> Value {
    json!({
        "angle": 0,
        "backgroundColor": background,
        "boundElements": [],
        "fillStyle": "hachure",
        "frameId": null,
        "height": num::json(rect.h),
        "id": id,
        "isDeleted": false,
        "locked": false,
        "opacity": 100,
        "roughness": 1,
        "roundness": null,
        "seed": seed & 0x7fff_ffff,
        "strokeColor": stroke,
        "strokeStyle": "solid",
        "strokeWidth": 2.0,
        "type": kind,
        "updated": 0,
        "version": 1,
        "versionNonce": 0,
        "width": num::json(rect.w),
        "x": num::json(rect.x),
        "y": num::json(rect.y),
    })
}

pub(super) fn color(theme: &Theme, token: Token) -> &'static str {
    theme.hex(token).unwrap_or("transparent")
}

/// Código numérico de `fontFamily` no Excalidraw (2 = Helvetica, 5 = Excalifont).
const fn family_code(family: FontFamily) -> u8 {
    match family {
        FontFamily::Hand => 5,
        FontFamily::Sans => 2,
    }
}

pub(super) fn text_element(
    id: &str,
    text: &str,
    rect: Rect,
    color: &'static str,
    fontsize: f32,
    container: Option<&str>,
    family: FontFamily,
) -> Value {
    json!({
        "angle": 0,
        "backgroundColor": "transparent",
        "baseline": num::json(fontsize * 0.9),
        "boundElements": [],
        "containerId": container,
        "fillStyle": "hachure",
        "fontFamily": family_code(family),
        "fontSize": num::json(fontsize),
        "frameId": null,
        "groupIds": [],
        "height": num::json(fontsize * 1.25),
        "id": id,
        "isDeleted": false,
        "lineHeight": 1.25,
        "locked": false,
        "opacity": 100,
        "originalText": text,
        "roughness": 1,
        "roundness": null,
        "seed": seed_for(id) & 0x7fff_ffff,
        "strokeColor": color,
        "strokeStyle": "solid",
        "strokeWidth": 1.0,
        "text": text,
        "textAlign": "center",
        "type": "text",
        "updated": 0,
        "version": 1,
        "versionNonce": 0,
        "verticalAlign": "middle",
        "width": num::json(rect.w),
        "x": num::json(rect.x),
        "y": num::json(rect.y),
    })
}

pub(super) fn binding(placed: &Placed, target: &str, point: Option<Point>) -> Value {
    let Some((node, point)) = placed.node(target).zip(point) else {
        return Value::Null;
    };
    let ratio = |offset: f32, size: f32| -> f64 {
        if size <= f32::EPSILON {
            return 0.0;
        }
        let value = (f64::from(offset) / f64::from(size)).clamp(0.0, 1.0);
        if (value - 0.5).abs() < 0.0001 {
            0.5001
        } else {
            value
        }
    };
    json!({
        "elementId": element_id(&node.id),
        "fixedPoint": [
            ratio(point.0 - node.rect.x, node.rect.w),
            ratio(point.1 - node.rect.y, node.rect.h),
        ],
        "focus": 0.0,
        "gap": 4.0,
    })
}

/// Largura e altura da rota (o bbox do `arrow`).
pub(super) fn route_extent(route: &[Point]) -> (f32, f32) {
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;
    if let Some(first) = route.first().copied() {
        for point in route {
            width = width.max((point.0 - first.0).abs());
            height = height.max((point.1 - first.1).abs());
        }
    }
    (width, height)
}

pub(super) fn excalidraw_kind(shape: Shape) -> &'static str {
    match shape {
        Shape::Circle => "ellipse",
        Shape::Diamond => "diamond",
        Shape::Rounded
        | Shape::Rect
        | Shape::Cloud
        | Shape::Cylinder
        | Shape::Hexagon
        | Shape::Note
        | Shape::Text => "rectangle",
    }
}

pub(super) fn fill_style(name: &str) -> &'static str {
    match name {
        "solid" => "solid",
        "cross-hatch" => "cross-hatch",
        "zigzag" => "zigzag",
        _ => "hachure",
    }
}

pub(super) fn stroke_style(style: StrokeStyle) -> &'static str {
    match style {
        StrokeStyle::Solid => "solid",
        StrokeStyle::Dashed => "dashed",
        StrokeStyle::Dotted => "dotted",
    }
}
