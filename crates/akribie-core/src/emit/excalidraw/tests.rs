//! Testes do `.excalidraw`: determinismo e invariantes do formato.

use serde_json::Value;

use super::excalidraw;
use crate::config::Config;
use crate::layout::layout;
use crate::parse::parse;
use crate::resolve::resolve;
use crate::text::Font;

const SAMPLE: &str = "chart v1 \"Retry\"\nnode api \"API\"\nnode db \"DB\" shape=circle fill=blue\nedge api -> db \"q\"\nnote \"n\" near=api\n";

fn document_of(source: &str) -> Value {
    let font = Font::load().expect("fonte");
    let (document, _) = parse(source);
    let (chart, _) = resolve(&document, &Config::default());
    let placed = layout(&chart, &font);
    serde_json::from_str(&excalidraw(&chart, &placed)).expect("json")
}

#[test]
fn export_is_deterministic() {
    let font = Font::load().expect("fonte");
    let (document, _) = parse(SAMPLE);
    let (chart, _) = resolve(&document, &Config::default());
    let placed = layout(&chart, &font);
    assert_eq!(excalidraw(&chart, &placed), excalidraw(&chart, &placed));
}

#[test]
fn elements_are_ordered_and_indexed() {
    let json = document_of(SAMPLE);
    assert_eq!(json["type"], "excalidraw");
    let elements = json["elements"].as_array().expect("elements");
    assert!(elements.len() >= 5);
    let indexes: Vec<String> = elements
        .iter()
        .map(|element| element["index"].as_str().unwrap_or("").to_string())
        .collect();
    for window in indexes.windows(2) {
        assert!(window[0] < window[1], "{indexes:?}");
    }
    for index in &indexes {
        assert!(!index.ends_with('0'), "{index}");
    }
}

#[test]
fn node_text_is_bound_to_its_container() {
    let json = document_of(SAMPLE);
    let elements = json["elements"].as_array().expect("elements");
    let node = elements
        .iter()
        .find(|element| element["id"] == "akr-api")
        .expect("nó");
    let bound = node["boundElements"].as_array().expect("bound");
    let text_id = bound[0]["id"].as_str().expect("id");
    let text = elements
        .iter()
        .find(|element| element["id"] == text_id)
        .expect("texto");
    assert_eq!(text["containerId"], "akr-api");
    assert_eq!(text["text"], "API");
}

#[test]
fn bindings_exist_and_never_use_exact_half() {
    let json = document_of(SAMPLE);
    let elements = json["elements"].as_array().expect("elements");
    let arrow = elements
        .iter()
        .find(|element| element["type"] == "arrow")
        .expect("seta");
    for side in ["startBinding", "endBinding"] {
        let fixed = arrow[side]["fixedPoint"].as_array().expect("fixedPoint");
        for value in fixed {
            let number = value.as_f64().unwrap_or(0.0);
            assert!((number - 0.5).abs() > 1e-6, "{side}: {number}");
            assert!((0.0..=1.0).contains(&number), "{side}: {number}");
        }
    }
}
