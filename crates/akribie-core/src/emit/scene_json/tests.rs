//! Testes do `scene` JSON: determinismo, forma e ausência de `path`.

use super::scene_json;
use crate::config::Config;
use crate::layout::layout;
use crate::parse::parse;
use crate::resolve::resolve;
use crate::text::Font;

fn scene_of(source: &str) -> String {
    let font = Font::load().expect("fonte");
    let (document, _) = parse(source);
    let (chart, _) = resolve(&document, &Config::default());
    let placed = layout(&chart, &font);
    scene_json(&placed, &chart)
}

const SAMPLE: &str = "chart v1 \"Retry\"\nnode api \"API\"\nnode db \"DB\"\nedge api -> db \"q\"\nnote \"n\" near=api\n";

#[test]
fn scene_json_is_deterministic() {
    assert_eq!(scene_of(SAMPLE), scene_of(SAMPLE));
}

#[test]
fn scene_json_describes_the_geometry() {
    let text = scene_of(SAMPLE);
    let value: serde_json::Value = serde_json::from_str(&text).expect("json válido");
    assert!(text.ends_with("}\n"));
    assert_eq!(value["layout"], "flow");
    assert_eq!(value["theme"], "paper");
    let nodes = value["nodes"].as_array().expect("nodes");
    assert_eq!(nodes.len(), 2);
    assert!(nodes[0]["rect"]["w"].as_f64().unwrap_or(0.0) > 0.0);
    assert_eq!(nodes[0]["label"], "API");
    let edges = value["edges"].as_array().expect("edges");
    assert_eq!(edges[0]["id"], "api->db#0");
    assert!(edges[0]["route"].as_array().map_or(0, Vec::len) >= 2);
    assert_eq!(value["notes"].as_array().map_or(0, Vec::len), 1);
}

#[test]
fn scene_json_has_no_path_strings() {
    let text = scene_of(SAMPLE);
    assert!(!text.contains("\"d\""), "{text}");
}

#[test]
fn numbers_are_rounded_to_two_places() {
    let text = scene_of(SAMPLE);
    for token in text.split(|c: char| !(c.is_ascii_digit() || c == '.')) {
        if token.contains('.') {
            let decimals = token.split('.').nth(1).unwrap_or("");
            assert!(decimals.len() <= 2, "número com excesso de casas: {token}");
        }
    }
}
