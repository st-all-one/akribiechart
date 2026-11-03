//! Contrato de borda de `akr import` (ADR-0004): `.excalidraw` → `.akrc` canônico.

mod common;

use common::{run, temp_file};

const MINIMAL: &str = r##"{
  "type": "excalidraw",
  "elements": [
    {"id": "box1", "type": "rectangle", "x": 10, "y": 10, "width": 120, "height": 50,
     "backgroundColor": "transparent", "strokeColor": "#1e1e1e", "roundness": {"type": 3}},
    {"id": "box2", "type": "ellipse", "x": 300, "y": 10, "width": 100, "height": 50,
     "backgroundColor": "transparent", "strokeColor": "#1971c2"},
    {"id": "l1", "type": "text", "containerId": "box1", "x": 12, "y": 18,
     "width": 40, "height": 20, "text": "API"},
    {"id": "a1", "type": "arrow", "x": 130, "y": 35, "width": 170, "height": 0,
     "startBinding": {"elementId": "box1"}, "endBinding": {"elementId": "box2"}},
    {"id": "freetext", "type": "text", "x": 10, "y": 120, "width": 80, "height": 20,
     "text": "nota solta"}
  ]
}"##;

#[test]
fn excalidraw_becomes_canonical_akrc_accepted_by_check() {
    let imported = run(&["import", "-"], Some(MINIMAL));
    assert_eq!(imported.code, 0, "stderr: {}", imported.stderr);
    assert!(
        imported.stdout.starts_with("chart v1"),
        "{}",
        imported.stdout
    );
    assert!(
        imported.stdout.contains("node box1 \"API\"") && imported.stdout.contains("shape=rounded"),
        "{}",
        imported.stdout
    );
    assert!(
        imported.stdout.contains("shape=circle"),
        "{}",
        imported.stdout
    );
    assert!(
        imported.stdout.contains("edge box1 -> box2"),
        "{}",
        imported.stdout
    );
    assert!(
        imported.stdout.contains("note \"nota solta\""),
        "{}",
        imported.stdout
    );
    let checked = run(&["check", "-"], Some(&imported.stdout));
    assert_eq!(
        checked.code, 0,
        "stdout: {} / stderr: {}",
        checked.stdout, checked.stderr
    );
}

#[test]
fn import_from_a_file_keeps_the_diagnostics_in_the_envelope() {
    let input = temp_file("import-envelope.excalidraw", MINIMAL);
    let run = run(&["import", input.to_str().expect("utf8"), "--json"], None);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(run.stdout.contains("\"source\""), "{}", run.stdout);
    assert!(run.stdout.contains("\"diagnostics\""), "{}", run.stdout);
}

#[test]
fn invalid_json_is_invalid_input() {
    let run = run(&["import", "-"], Some("isto não é JSON"));
    assert_eq!(run.code, 2, "stderr: {}", run.stderr);
    assert!(run.stderr.contains("JSON inválido"), "{}", run.stderr);
}

#[test]
fn wrong_document_shape_is_invalid_input() {
    let run = run(&["import", "-"], Some("{\"type\":\"excalidraw\"}"));
    assert_eq!(run.code, 2, "stderr: {}", run.stderr);
    assert!(run.stderr.contains("elements"), "{}", run.stderr);
}

#[test]
fn params_choose_the_source_and_the_title() {
    let path = temp_file("desenho.excalidraw", MINIMAL);
    let quoted = serde_json::to_string(path.to_str().unwrap_or_default()).unwrap_or_default();
    let params = format!("{{\"path\":{quoted},\"name\":\"Retry\"}}");
    let run = run(&["import", "--params", &params], None);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(
        run.stdout.starts_with("chart v1 \"Retry\""),
        "{}",
        run.stdout
    );
}

#[test]
fn params_plus_positional_is_usage_error() {
    let path = temp_file("p.excalidraw", MINIMAL);
    let run = run(
        &[
            "import",
            path.to_str().unwrap_or_default(),
            "--params",
            "{}",
        ],
        None,
    );
    assert_eq!(run.code, 2, "stderr: {}", run.stderr);
}
