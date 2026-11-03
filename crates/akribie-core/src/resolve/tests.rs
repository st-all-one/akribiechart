//! Testes da resolução semântica.

use super::resolve;
use crate::config::Config;
use crate::diag::Diagnostic;
use crate::model::{Chart, Curve, Direction, Head, Layout, Shape};
use crate::parse::parse;
use crate::theme::Token;

fn chart_of(source: &str) -> (Chart, Vec<Diagnostic>) {
    chart_with(source, &Config::default())
}

fn chart_with(source: &str, config: &Config) -> (Chart, Vec<Diagnostic>) {
    let (document, mut diagnostics) = parse(source);
    let (chart, resolved) = resolve(&document, config);
    diagnostics.extend(resolved);
    (chart, diagnostics)
}

fn codes(diagnostics: &[Diagnostic]) -> Vec<&'static str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.code)
        .collect()
}

const SAMPLE: &str =
    "chart v1 \"Retry\" layout=tree direction=tb theme=slate fontsize=24 pad=16 lang=pt-BR
node api \"API\" shape=circle fill=blue
node db \"Banco\" fill=green fillstyle=solid
edge api -> db \"lê\" curve=elbow head=none tail=arrow style=dashed width=4
";

#[test]
fn options_reach_the_model() {
    let (chart, diagnostics) = chart_of(SAMPLE);
    assert_eq!(codes(&diagnostics), Vec::<&str>::new());
    assert_eq!(chart.layout, Layout::Tree);
    assert_eq!(chart.direction, Direction::Tb);
    assert_eq!(chart.theme.name, "slate");
    assert_eq!(chart.fontsize, 24.0);
    assert_eq!(chart.pad, 16.0);
    assert_eq!(chart.lang, "pt-BR");
    let api = chart.node("api").expect("api");
    assert_eq!(api.label, "API");
    assert_eq!(api.shape, Shape::Circle);
    assert_eq!(api.fill, Token::Blue);
    let edge = chart.edges.first().expect("aresta");
    assert_eq!(edge.curve, Curve::Elbow);
    assert_eq!(edge.head, Head::None);
    assert_eq!(edge.tail, Head::Arrow);
    assert_eq!(edge.width, 4.0);
}

#[test]
fn defaults_come_from_config_when_chart_omits_them() {
    let config = Config {
        layout: Layout::Grid,
        fontsize: 30.0,
        ..Config::default()
    };
    let (chart, diagnostics) = chart_with("chart v1 \"x\"\n", &config);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(chart.layout, Layout::Grid);
    assert_eq!(chart.fontsize, 30.0);
}

#[test]
fn chart_options_win_over_config() {
    let config = Config {
        layout: Layout::Grid,
        ..Config::default()
    };
    let (chart, _) = chart_with("chart v1 \"x\" layout=stack\n", &config);
    assert_eq!(chart.layout, Layout::Stack);
}

#[test]
fn implicit_nodes_are_created_once_in_file_order() {
    let (chart, diagnostics) = chart_of("chart v1 \"x\"\nedge a -> b\nedge b -> a\nedge a -> c\n");
    assert_eq!(codes(&diagnostics), ["W001", "W001", "W001"]);
    let ids: Vec<&str> = chart.nodes.iter().map(|node| node.id.as_str()).collect();
    assert_eq!(ids, ["a", "b", "c"]);
    assert!(chart.nodes.iter().all(|node| node.implicit));
    assert_eq!(chart.edges.len(), 3);
}

#[test]
fn implicit_node_reuses_declared_label() {
    let (chart, diagnostics) = chart_of("chart v1 \"x\"\nnode a \"A\"\nedge a -> b\n");
    assert_eq!(codes(&diagnostics), ["W001"]);
    let b = chart.node("b").expect("b");
    assert_eq!(b.label, "b");
}

#[test]
fn duplicate_id_is_an_error_and_the_second_is_dropped() {
    let (chart, diagnostics) = chart_of("chart v1 \"x\"\nnode a \"A\"\nnode a \"B\"\n");
    assert_eq!(codes(&diagnostics), ["E004"]);
    assert_eq!(chart.nodes.len(), 1);
    assert_eq!(chart.node("a").map(|node| node.label.as_str()), Some("A"));
}

#[test]
fn edge_to_a_group_is_missing_reference() {
    let (chart, diagnostics) = chart_of("chart v1 \"x\"\ngroup g \"G\"\nnode a\nedge a -> g\n");
    assert_eq!(codes(&diagnostics), ["E005"]);
    assert_eq!(chart.edges.len(), 0);
}

#[test]
fn in_requires_a_declared_group() {
    let (_, diagnostics) = chart_of("chart v1 \"x\"\nnode a in=g\nnode b in=z\ngroup g \"G\"\n");
    assert_eq!(codes(&diagnostics), ["E005"]);
}

#[test]
fn free_layout_requires_positions_on_nodes_and_notes() {
    let (_, diagnostics) = chart_of(
        "chart v1 \"x\" layout=free\nnode a\nnode b at=10,20\nnote \"n\"\nnote \"m\" at=30,40\n",
    );
    assert_eq!(codes(&diagnostics), ["E006", "E006"]);
}

#[test]
fn edges_from_the_same_pair_get_increasing_ordinals() {
    let (chart, _) =
        chart_of("chart v1 \"x\"\nnode a\nnode b\nedge a -> b\nedge a -> b\nedge b -> a\n");
    let ordinals: Vec<usize> = chart.edges.iter().map(|edge| edge.ordinal).collect();
    assert_eq!(ordinals, [0, 1, 0]);
}

#[test]
fn note_near_must_be_a_node() {
    let (chart, diagnostics) = chart_of("chart v1 \"x\"\ngroup g \"G\"\nnote \"n\" near=g\n");
    assert_eq!(codes(&diagnostics), ["E005"]);
    assert_eq!(
        chart.notes.first().and_then(|note| note.near.as_deref()),
        None
    );
}

#[test]
fn group_membership_is_kept_as_ids() {
    let (chart, diagnostics) = chart_of("chart v1 \"x\"\ngroup g \"G\"\nnode a in=g\n");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(
        chart.node("a").map(|node| node.groups.clone()),
        Some(vec!["g".to_string()])
    );
}

#[test]
fn no_chart_statement_still_resolves() {
    let (chart, diagnostics) = chart_of("node a\n");
    assert_eq!(codes(&diagnostics), ["E008"]);
    assert_eq!(chart.nodes.len(), 1);
}

#[test]
fn sketch_and_font_follow_the_chart_then_the_config() {
    use crate::model::FontFamily;
    let config = Config::from_toml("sketch = off\nfont = sans\n").expect("config");
    let (chart, _) = chart_with("chart v1 \"x\"\n", &config);
    assert!(!chart.sketch);
    assert_eq!(chart.font, FontFamily::Sans);
    let (chart, _) = chart_with("chart v1 \"x\" sketch=on font=hand\n", &config);
    assert!(chart.sketch);
    assert_eq!(chart.font, FontFamily::Hand);
    let (chart, _) = chart_of("chart v1 \"x\"\n");
    assert!(chart.sketch, "o padrão é traço à mão");
    assert_eq!(chart.font, FontFamily::Hand);
}
