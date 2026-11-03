//! Testes de pintura: folga dos rótulos de aresta e caixa segura do texto.

use crate::config::Config;
use crate::geometry::Rect;
use crate::layout::{Placed, layout, text_box};
use crate::model::{Chart, Shape};
use crate::paint::{LABEL_PAD, Scene, paint};
use crate::pipeline::check;
use crate::text::Font;

/// Analisa, resolve, posiciona e pinta — o caminho real de `render`.
fn build(source: &str) -> (Chart, Placed, Scene, Font) {
    let font = Font::load().expect("fontes");
    let report = check(source, &Config::default(), &font);
    assert!(
        report.diagnostics.is_empty(),
        "fonte de teste precisa estar limpa: {:?}",
        report.diagnostics
    );
    let placed = layout(&report.chart, &font);
    let scene = paint(&report.chart, &placed, &font);
    (report.chart, placed, scene, font)
}

fn distance_to_rect(point: (f32, f32), rect: Rect) -> f32 {
    let dx = (rect.x - point.0).max(point.0 - rect.right()).max(0.0);
    let dy = (rect.y - point.1).max(point.1 - rect.bottom()).max(0.0);
    dx.hypot(dy)
}

/// Menor distância entre a rota (amostrada por segmento) e o retângulo do rótulo.
fn route_clearance(route: &[(f32, f32)], rect: Rect) -> f32 {
    const SAMPLES: u8 = 32;
    let mut minimum = f32::INFINITY;
    for pair in route.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        for step in 0..=SAMPLES {
            let t = f32::from(step) / f32::from(SAMPLES);
            let point = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
            minimum = minimum.min(distance_to_rect(point, rect));
        }
    }
    minimum
}

#[test]
fn straight_edge_label_keeps_the_padding_from_its_line() {
    let (_, placed, scene, _) = build(
        "chart v1 \"x\" layout=free\nnode a \"A\" at=0,0\nnode b \"B\" at=300,0\nedge a -> b \"meio\"\n",
    );
    for (edge, drawn) in placed.edges.iter().zip(&scene.edges) {
        let Some(label) = &drawn.label else { continue };
        let clearance = route_clearance(&edge.route, label.rect);
        assert!(clearance >= LABEL_PAD - 0.01, "{clearance} < {LABEL_PAD}");
    }
}

#[test]
fn curved_edge_label_keeps_the_padding_from_its_line() {
    let (_, placed, scene, _) = build(
        "chart v1 \"x\" layout=free\nnode a \"A\" at=0,0\nnode b \"B\" at=300,120\nedge a -> b \"curva\" curve=arc\n",
    );
    let edge = placed.edges.first().expect("aresta");
    let label = scene
        .edges
        .first()
        .and_then(|drawn| drawn.label.as_ref())
        .expect("rótulo");
    let clearance = route_clearance(&edge.route, label.rect);
    assert!(clearance >= LABEL_PAD - 0.01, "{clearance} < {LABEL_PAD}");
}

#[test]
fn node_text_lives_inside_the_safe_box_for_every_shape() {
    for shape in [
        Shape::Rect,
        Shape::Rounded,
        Shape::Circle,
        Shape::Diamond,
        Shape::Cloud,
        Shape::Cylinder,
        Shape::Hexagon,
        Shape::Note,
    ] {
        let source = format!(
            "chart v1 \"x\" layout=grid\nnode a \"Rótulo comprido\" shape={}\n",
            shape.name()
        );
        let (_, placed, scene, _) = build(&source);
        let placed_node = placed.nodes.first().expect("nó posicionado");
        let drawn = scene.nodes.first().expect("nó pintado");
        let box_ = text_box(placed_node.shape, placed_node.rect);
        let lines = drawn.text.len();
        assert_eq!(lines, placed_node.label.len());
        for run in &drawn.text {
            let half = run.size * 0.5;
            assert!(
                run.y - half >= box_.y - 0.01,
                "{}: topo do texto acima da caixa segura",
                shape.name()
            );
            assert!(
                run.y + half <= box_.bottom() + 0.01,
                "{}: base do texto abaixo da caixa segura",
                shape.name()
            );
        }
    }
}

#[test]
fn clean_finish_uses_a_single_pass() {
    let (_, _, scene, _) =
        build("chart v1 \"x\" sketch=off\nnode a \"A\"\nnode b \"B\"\nedge a -> b \"x\"\n");
    let node = scene.nodes.first().expect("nó");
    let stroke = node.stroke.as_ref().expect("contorno");
    assert_eq!(stroke.d.matches('M').count(), 1, "{}", stroke.d);
    assert!(stroke.d.contains('A'), "{}", stroke.d);
}

#[test]
fn hand_finish_uses_two_passes() {
    let (_, _, scene, _) = build("chart v1 \"x\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b \"x\"\n");
    let node = scene.nodes.first().expect("nó");
    let stroke = node.stroke.as_ref().expect("contorno");
    assert_eq!(stroke.d.matches('M').count(), 2, "{}", stroke.d);
}
