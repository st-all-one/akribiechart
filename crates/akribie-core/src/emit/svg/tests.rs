//! Testes do SVG: estrutura acessível, escaping e determinismo.

use super::svg;
use crate::config::Config;
use crate::layout::layout;
use crate::model::Chart;
use crate::paint::{Scene, paint};
use crate::parse::parse;
use crate::resolve::resolve;
use crate::text::Font;

fn scene_of(source: &str) -> (Chart, Scene) {
    let font = Font::load().expect("fonte");
    let (document, _) = parse(source);
    let (chart, _) = resolve(&document, &Config::default());
    let placed = layout(&chart, &font);
    let scene = paint(&chart, &placed, &font);
    (chart, scene)
}

#[test]
fn svg_is_byte_identical_for_the_same_scene() {
    let (chart, scene) = scene_of(
        "chart v1 \"T\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b \"x\"\nnote \"n\" near=a\n",
    );
    assert_eq!(svg(&scene, chart.theme), svg(&scene, chart.theme));
}

#[test]
fn svg_has_the_accessible_structure() {
    let (chart, scene) =
        scene_of("chart v1 \"Retry\" lang=pt-BR\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n");
    let document = svg(&scene, chart.theme);
    for expected in [
        "role=\"img\"",
        "<title id=\"akr-t\">Retry</title>",
        "<desc id=\"akr-d\">",
        "aria-labelledby=\"akr-t akr-d\"",
        "lang=\"pt-BR\"",
        "focusable=\"false\"",
    ] {
        assert!(document.contains(expected), "faltou {expected}: {document}");
    }
    assert!(document.contains("<g id=\"edges\">"));
    assert!(document.contains("<g id=\"nodes\">"));
    assert!(document.contains("data-kind=\"edge\""));
}

#[test]
fn markup_is_escaped() {
    let (chart, scene) = scene_of("chart v1 \"A & B <x>\"\nnode a \"a & b\"\n");
    let document = svg(&scene, chart.theme);
    assert!(document.contains("A &amp; B &lt;x&gt;"), "{document}");
    assert!(document.contains("a &amp; b"));
}

#[test]
fn arrows_follow_the_declared_sides() {
    let (_, scene) = scene_of("chart v1 \"x\"\nnode a\nnode b\nedge a -> b head=arrow tail=none\n");
    let edge = scene.edges.first().expect("aresta");
    assert!(edge.head.is_some());
    assert!(edge.tail.is_none());
    let (_, scene) = scene_of("chart v1 \"x\"\nnode a\nnode b\nedge a -> b head=none tail=arrow\n");
    let edge = scene.edges.first().expect("aresta");
    assert!(edge.head.is_none());
    assert!(edge.tail.is_some());
}

#[test]
fn every_shape_produces_stroke_and_fill() {
    for shape in [
        "rect", "rounded", "circle", "diamond", "cloud", "cylinder", "hexagon", "note",
    ] {
        let (_, scene) = scene_of(&format!(
            "chart v1 \"x\"\nnode a \"A\" shape={shape} fill=blue\n"
        ));
        let node = scene.nodes.first().expect("nó");
        assert!(node.stroke.is_some(), "{shape}");
        assert!(node.fill.is_some(), "{shape}");
        assert!(node.clip.is_some(), "{shape} deve recortar a hachura");
    }
}
