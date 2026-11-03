//! Testes do `.excalidraw` → `.akrc`.

use super::import;
use crate::config::Config;
use crate::parse::parse;
use crate::resolve::resolve;

const SCENE: &str = r##"{
  "type": "excalidraw",
  "version": 2,
  "elements": [
    {"id":"box1","type":"rectangle","x":10,"y":20,"width":120,"height":50,
     "strokeColor":"#1e1e1e","backgroundColor":"#a5d8ff","strokeStyle":"solid","isDeleted":false},
    {"id":"t1","type":"text","x":30,"y":30,"width":80,"height":25,"text":"API",
     "containerId":"box1","isDeleted":false},
    {"id":"box2","type":"ellipse","x":300,"y":20,"width":100,"height":50,
     "strokeColor":"#1e1e1e","backgroundColor":"transparent","isDeleted":false},
    {"id":"a1","type":"arrow","x":130,"y":45,"width":170,"height":0,
     "startBinding":{"elementId":"box1"},"endBinding":{"elementId":"box2"},
     "strokeStyle":"dashed","isDeleted":false},
    {"id":"img1","type":"image","x":0,"y":0,"width":10,"height":10,"isDeleted":false}
  ]
}"##;

#[test]
fn maps_shapes_text_arrows_and_reports_losses() {
    let imported = import(SCENE, "diagrama").expect("importa");
    assert!(
        imported
            .source
            .starts_with("chart v1 \"diagrama\" layout=free\n")
    );
    assert!(imported.source.contains("node box1 \"API\""));
    assert!(imported.source.contains("shape=rounded"));
    assert!(imported.source.contains("fill=blue"));
    assert!(imported.source.contains("node box2"));
    assert!(imported.source.contains("shape=circle"));
    assert!(imported.source.contains("edge box1 -> box2"));
    assert!(imported.source.contains("style=dashed"));
    assert!(!imported.source.contains("note"));
    assert!(
        imported
            .diagnostics
            .iter()
            .any(|d| d.code.name == "element-dropped")
    );
}

#[test]
fn the_result_passes_check() {
    let imported = import(SCENE, "diagrama").expect("importa");
    let (document, errors) = parse(&imported.source);
    assert!(errors.is_empty(), "{errors:?}\n{}", imported.source);
    let (chart, diagnostics) = resolve(&document, &Config::default());
    assert!(
        !diagnostics.iter().any(|d| d.code.name != "implicit-node"),
        "{diagnostics:?}"
    );
    assert_eq!(chart.edges.len(), 1);
}

#[test]
fn unmapped_colors_warn() {
    let source = r##"{"type":"excalidraw","elements":[
        {"id":"b","type":"rectangle","x":0,"y":0,"width":10,"height":10,
         "backgroundColor":"#123456","strokeColor":"#abcdef"}]}"##;
    let imported = import(source, "x").expect("importa");
    let warnings: Vec<&str> = imported
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.name)
        .collect();
    assert_eq!(
        warnings
            .iter()
            .filter(|name| **name == "color-unmapped")
            .count(),
        2
    );
    assert!(imported.source.contains("fill=none"));
    assert!(imported.source.contains("stroke=muted"));
}

#[test]
fn free_text_becomes_a_note() {
    let source = "{\"type\":\"excalidraw\",\"elements\":[{\"id\":\"t\",\"type\":\"text\",\"x\":5,\"y\":6,\"width\":40,\"height\":20,\"text\":\"solto\"}]}";
    let imported = import(source, "x").expect("importa");
    assert!(
        imported.source.contains("note \"solto\""),
        "{}",
        imported.source
    );
}

#[test]
fn rejects_foreign_json() {
    assert!(import("{ \"type\": \"other\", \"elements\": [] }", "x").is_err());
    assert!(import("não é json", "x").is_err());
}
