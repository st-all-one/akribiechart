#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "teste: a asserção precisa falhar alto e apontar a linha"
)]

//! Contrato do parser: recuperação por linha e códigos de diagnóstico.

use akribie_core::diag::Diagnostic;
use akribie_core::parse::parse;
use akribie_core::syntax::OptVal;

fn codes(diagnostics: &[Diagnostic]) -> Vec<&'static str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.code)
        .collect()
}

#[test]
fn minimal_chart_parses_without_diagnostics() {
    let (document, diagnostics) = parse("chart v1 \"x\"\nnode a \"A\"\n");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let chart = document.chart.expect("chart presente");
    assert_eq!(chart.title.as_deref(), Some("x"));
    assert_eq!(document.nodes.len(), 1);
    assert_eq!(
        document.nodes.first().map(|node| node.id.as_str()),
        Some("a")
    );
    assert_eq!(
        document
            .nodes
            .first()
            .and_then(|node| node.label.as_deref()),
        Some("A")
    );
}

#[test]
fn first_statement_must_be_chart() {
    let (_, diagnostics) = parse("node a\nchart v1 \"x\"\n");
    assert_eq!(codes(&diagnostics), vec!["E008"]);
    assert_eq!(diagnostics.first().map(|d| d.span.line), Some(1));
}

#[test]
fn duplicate_chart_is_reported() {
    let (_, diagnostics) = parse("chart v1 \"x\"\nchart v1 \"y\"\n");
    assert_eq!(codes(&diagnostics), vec!["E008"]);
    assert_eq!(diagnostics.first().map(|d| d.span.line), Some(2));
}

#[test]
fn unsupported_version_is_reported() {
    let (_, diagnostics) = parse("chart v2 \"x\"\n");
    assert_eq!(codes(&diagnostics), vec!["E009"]);
}

#[test]
fn unknown_keyword_lists_valid_options() {
    let (_, diagnostics) = parse("chart v1 \"x\"\narrow a -> b\n");
    assert_eq!(codes(&diagnostics), vec!["E001"]);
    let hint = diagnostics
        .first()
        .and_then(|d| d.hint.clone())
        .unwrap_or_default();
    assert!(hint.contains("chart"), "{hint}");
}

#[test]
fn unknown_key_lists_valid_keys() {
    let (_, diagnostics) = parse("chart v1 \"x\"\nnode a fill=blue colour=red\n");
    assert_eq!(codes(&diagnostics), vec!["E002"]);
    let hint = diagnostics
        .first()
        .and_then(|d| d.hint.clone())
        .unwrap_or_default();
    assert!(hint.contains("fillstyle"), "{hint}");
}

#[test]
fn invalid_value_lists_closed_set() {
    let (_, diagnostics) = parse("chart v1 \"x\"\nnode a \"A\" shape=cilinder\n");
    assert_eq!(codes(&diagnostics), vec!["E003"]);
    let hint = diagnostics
        .first()
        .and_then(|d| d.hint.clone())
        .unwrap_or_default();
    assert!(hint.contains("cylinder"), "{hint}");
    assert_eq!(diagnostics.first().map(|d| d.span.line), Some(2));
}

#[test]
fn range_and_type_are_enforced() {
    let (_, diagnostics) = parse("chart v1 \"x\" fontsize=200\nnode a width=9\nnode b at=1\n");
    assert_eq!(codes(&diagnostics), vec!["E003", "E003", "E003"]);
}

#[test]
fn tab_is_malformed_and_does_not_break_next_line() {
    let (document, diagnostics) = parse("chart v1 \"x\"\nnode\ta\nnode b\n");
    assert_eq!(codes(&diagnostics), vec!["E007"]);
    assert_eq!(document.nodes.len(), 1);
    assert_eq!(
        document.nodes.first().map(|node| node.id.as_str()),
        Some("b")
    );
}

#[test]
fn edge_without_arrow_is_malformed() {
    let (_, diagnostics) = parse("chart v1 \"x\"\nedge a b\n");
    assert_eq!(codes(&diagnostics), vec!["E007"]);
    let hint = diagnostics
        .first()
        .and_then(|d| d.hint.clone())
        .unwrap_or_default();
    assert!(hint.contains("->"), "{hint}");
}

#[test]
fn comments_travel_with_the_next_statement() {
    let (document, diagnostics) =
        parse("# topo\nchart v1 \"x\"\n\n# sobre a\tnode a\nnode b\n# solto\n");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let chart = document.chart.expect("chart");
    assert_eq!(chart.comments, vec!["# topo".to_string()]);
    assert_eq!(
        document.nodes.first().map(|node| node.comments.clone()),
        Some(vec!["# sobre a\tnode a".to_string()])
    );
    assert_eq!(document.trailing_comments, vec!["# solto".to_string()]);
}

#[test]
fn options_are_typed_and_ordered() {
    let (document, diagnostics) =
        parse("chart v1 \"x\"\nnode a size=120,40 at=1.5,-2 fill=amber in=g1,g2\n");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let node = document.nodes.first().expect("node");
    assert_eq!(node.options.get("size"), Some(&OptVal::Pair(120.0, 40.0)));
    assert_eq!(node.options.get("at"), Some(&OptVal::Pair(1.5, -2.0)));
    assert_eq!(
        node.options.get("in"),
        Some(&OptVal::Ids(vec!["g1".to_string(), "g2".to_string()]))
    );
    let keys: Vec<&str> = node.options.keys().copied().collect();
    assert_eq!(keys, vec!["size", "at", "fill", "in"]);
}

#[test]
fn nodes_groups_edges_and_notes_are_kept_in_order() {
    let source =
        "chart v1 \"x\"\ngroup g \"G\"\nnode a\nnode b\nedge a -> b \"ir\"\nnote \"nota\"\n";
    let (document, diagnostics) = parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(document.groups.len(), 1);
    assert_eq!(document.nodes.len(), 2);
    assert_eq!(document.edges.len(), 1);
    assert_eq!(document.notes.len(), 1);
    let edge = document.edges.first().expect("edge");
    assert_eq!((edge.from.as_str(), edge.to.as_str()), ("a", "b"));
    assert_eq!(edge.label.as_deref(), Some("ir"));
}
