//! Setas com bindings, rota e rótulo.

use serde_json::{Value, json};

use super::support::{
    binding, color, element_id, route_extent, set, stroke_style, tag, text_element,
};
use crate::geometry::Rect;
use crate::layout::{Placed, PlacedEdge};
use crate::model::Curve;
use crate::model::FontFamily;
use crate::num;
use crate::sketch::seed_for;
use crate::theme::Theme;

pub(super) fn push_edge(
    elements: &mut Vec<Value>,
    edge: &PlacedEdge,
    placed: &Placed,
    theme: &Theme,
    fontsize: f32,
    family: FontFamily,
) {
    let Some(first) = edge.route.first().copied() else {
        return;
    };
    let points: Vec<Value> = edge
        .route
        .iter()
        .map(|point| json!([num::json(point.0 - first.0), num::json(point.1 - first.1)]))
        .collect();
    let id = element_id(&format!("{}->{}#{}", edge.from, edge.to, edge.ordinal));
    let mut arrow = json!({
        "angle": 0,
        "backgroundColor": "transparent",
        "boundElements": [],
        "elbowed": false,
        "endArrowhead": if edge.head { Value::from("arrow") } else { Value::Null },
        "endBinding": binding(placed, &edge.to, edge.route.last().copied()),
        "fillStyle": "hachure",
        "frameId": null,
        "height": num::json(route_extent(&edge.route).1),
        "id": id,
        "isDeleted": false,
        "lastCommittedPoint": null,
        "locked": false,
        "opacity": 100,
        "points": points,
        "roughness": 1,
        "roundness": null,
        "seed": seed_for(&id) & 0x7fff_ffff,
        "startArrowhead": if edge.tail { Value::from("arrow") } else { Value::Null },
        "startBinding": binding(placed, &edge.from, edge.route.first().copied()),
        "strokeColor": color(theme, edge.stroke),
        "strokeStyle": stroke_style(edge.style),
        "strokeWidth": num::json(edge.width),
        "type": "arrow",
        "updated": 0,
        "version": 1,
        "versionNonce": 0,
        "width": num::json(route_extent(&edge.route).0),
        "x": num::json(first.0),
        "y": num::json(first.1),
    });
    if edge.curve == Curve::Arc {
        set(&mut arrow, "roundness", json!({ "type": 2 }));
    }
    tag(&mut arrow, "edge");
    if let Some(label) = &edge.label {
        let text_id = format!("{id}-label");
        set(
            &mut arrow,
            "boundElements",
            json!([{ "id": text_id, "type": "text" }]),
        );
        let width = f32::from(u16::try_from(label.chars().count()).unwrap_or(1)) * fontsize * 0.55;
        let height = fontsize * 1.25;
        let rect = Rect::new(
            edge.label_at.0 - width * 0.5,
            edge.label_at.1 - height * 0.5,
            width.max(20.0),
            height,
        );
        elements.push(arrow);
        elements.push(text_element(
            &text_id,
            label,
            rect,
            theme.ink,
            fontsize,
            Some(&id),
            family,
        ));
        return;
    }
    elements.push(arrow);
}
