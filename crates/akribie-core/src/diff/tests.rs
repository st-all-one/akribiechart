//! Testes do diff semântico (ordem, identidade e contagem de linhas).

use super::{Kind, diff};
use crate::config::Config;
use crate::model::Chart;
use crate::parse::parse;
use crate::resolve::resolve;

fn chart_of(source: &str) -> Chart {
    let (document, _) = parse(source);
    let (chart, _) = resolve(&document, &Config::default());
    chart
}

fn lines(old: &str, new: &str) -> Vec<String> {
    diff(&chart_of(old), &chart_of(new))
        .iter()
        .map(super::Change::line)
        .collect()
}

#[test]
fn identical_documents_have_no_diff() {
    let source = "chart v1 \"T\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n";
    assert_eq!(lines(source, source).len(), 0);
}

#[test]
fn three_changes_are_exactly_three_lines_in_order() {
    let old = "chart v1 \"T\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n";
    let new =
        "chart v1 \"T\"\nnode a \"A2\"\nnode b \"B\"\nnode c \"C\"\nedge a -> b\nedge b -> c\n";
    let lines = lines(old, new);
    assert_eq!(lines.len(), 3, "{lines:?}");
    assert_eq!(lines[0], "+ edge b->c");
    assert_eq!(lines[1], "+ node c \"C\"");
    assert_eq!(lines[2], "~ node a label: A -> A2");
}

#[test]
fn edges_match_by_from_to_and_ordinal() {
    let old = "chart v1 \"T\"\nnode a\nnode b\nedge a -> b \"1\"\nedge a -> b \"2\"\n";
    let new = "chart v1 \"T\"\nnode a\nnode b\nedge a -> b \"1\"\nedge a -> b \"3\"\n";
    let changes = diff(&chart_of(old), &chart_of(new));
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].kind, Kind::Changed);
    assert_eq!(changes[0].element, "edge a->b#1");
    assert_eq!(changes[0].before.as_deref(), Some("2"));
}

#[test]
fn chart_level_fields_are_reported() {
    let old = "chart v1 \"T\" theme=paper\nnode a \"A\"\n";
    let new = "chart v1 \"T2\" theme=slate\nnode a \"A\"\n";
    let lines = lines(old, new);
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines.iter().all(|line| line.starts_with("~ chart ")));
}

#[test]
fn notes_are_matched_by_anchor_and_position() {
    let old = "chart v1 \"T\"\nnode a\nnote \"x\" near=a\n";
    let new = "chart v1 \"T\"\nnode a\nnote \"y\" near=a\n";
    let changes = diff(&chart_of(old), &chart_of(new));
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].kind, Kind::Changed);
    assert_eq!(changes[0].field, Some("text"));
}
