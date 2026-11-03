//! Testes do layout: perfis, grupos, notas, rotas e determinismo.

use super::{Placed, layout};
use crate::config::Config;
use crate::model::Chart;
use crate::parse::parse;
use crate::resolve::resolve;
use crate::text::Font;

fn font() -> Font {
    Font::load().expect("fonte embutida")
}

fn chart_of(source: &str) -> Chart {
    let (document, _) = parse(source);
    let (chart, _) = resolve(&document, &Config::default());
    chart
}

pub(super) fn placed(source: &str) -> Placed {
    layout(&chart_of(source), &font())
}

fn centers(placed: &Placed) -> Vec<(String, f32, f32)> {
    placed
        .nodes
        .iter()
        .map(|node| {
            let (x, y) = node.rect.center();
            (node.id.clone(), x, y)
        })
        .collect()
}

#[test]
fn flow_lr_orders_the_chain_left_to_right_on_one_row() {
    let placed = placed(
        "chart v1 \"x\" layout=flow direction=lr\nnode a \"A\"\nnode b \"B\"\nnode c \"C\"\nedge a -> b\nedge b -> c\n",
    );
    let centers = centers(&placed);
    let x: Vec<f32> = centers.iter().map(|(_, x, _)| *x).collect();
    let y: Vec<f32> = centers.iter().map(|(_, _, y)| *y).collect();
    assert!(x[0] < x[1] && x[1] < x[2], "{x:?}");
    assert!((y[0] - y[2]).abs() < 0.5, "{y:?}");
}

#[test]
fn flow_tb_orders_top_to_bottom() {
    let placed = placed(
        "chart v1 \"x\" layout=flow direction=tb\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n",
    );
    let centers = centers(&placed);
    assert!(centers[0].2 < centers[1].2, "{centers:?}");
}

#[test]
fn grid_with_four_nodes_is_two_by_two() {
    let placed = placed(
        "chart v1 \"x\" layout=grid\nnode a \"A\"\nnode b \"B\"\nnode c \"C\"\nnode d \"D\"\n",
    );
    let xs: Vec<f32> = placed.nodes.iter().map(|node| node.rect.x).collect();
    let ys: Vec<f32> = placed.nodes.iter().map(|node| node.rect.y).collect();
    assert_eq!(
        (distinct_count(&xs), distinct_count(&ys)),
        (2, 2),
        "{xs:?} {ys:?}"
    );
}

#[test]
fn stack_is_a_single_column() {
    let placed = placed("chart v1 \"x\" layout=stack\nnode a \"A\"\nnode b \"B\"\nnode c \"C\"\n");
    let xs: Vec<f32> = placed.nodes.iter().map(|node| node.rect.x).collect();
    assert_eq!(distinct_count(&xs), 1, "{xs:?}");
    let ys: Vec<f32> = placed.nodes.iter().map(|node| node.rect.y).collect();
    assert!(ys.windows(2).all(|pair| pair[0] < pair[1]), "{ys:?}");
}

#[test]
fn tree_centers_the_parent_over_the_children() {
    let placed = placed(
        "chart v1 \"x\" layout=tree direction=tb\nnode r \"R\"\nnode a \"A\"\nnode b \"B\"\nedge r -> a\nedge r -> b\n",
    );
    let root = placed.node("r").expect("r").rect.center().0;
    let left = placed.node("a").expect("a").rect.center().0;
    let right = placed.node("b").expect("b").rect.center().0;
    assert!(
        (root - f32::midpoint(left, right)).abs() < 0.5,
        "{root} {left} {right}"
    );
    let root_y = placed.node("r").expect("r").rect.center().1;
    assert!(root_y < placed.node("a").expect("a").rect.center().1);
}

#[test]
fn mindmap_puts_children_on_a_ring() {
    let placed = placed(
        "chart v1 \"x\" layout=mindmap\nnode r \"R\"\nnode a \"A\"\nnode b \"B\"\nedge r -> a\nedge r -> b\n",
    );
    let (rx, ry) = placed.node("r").expect("r").rect.center();
    for id in ["a", "b"] {
        let (x, y) = placed.node(id).expect(id).rect.center();
        let radius = ((x - rx).powi(2) + (y - ry).powi(2)).sqrt();
        assert!((radius - 180.0).abs() < 1.0, "{id}: {radius}");
    }
}

#[test]
fn free_preserves_declared_offsets_and_sizes() {
    let placed = placed(
        "chart v1 \"x\" layout=free\nnode a \"A\" at=100,200 size=80,40\nnode b \"B\" at=400,100\nedge a -> b\n",
    );
    let a = placed.node("a").expect("a");
    let b = placed.node("b").expect("b");
    // `at` é relativo: o bbox é normalizado, os deslocamentos declarados são exatos.
    assert!(
        (b.rect.x - a.rect.x - 300.0).abs() < 0.01,
        "{:?} {:?}",
        a.rect,
        b.rect
    );
    assert!(
        (a.rect.y - b.rect.y - 100.0).abs() < 0.01,
        "{:?} {:?}",
        a.rect,
        b.rect
    );
    assert_eq!((a.rect.w, a.rect.h), (80.0, 40.0));
    assert!(a.rect.x >= 32.0 && a.rect.y >= 32.0);
}

#[test]
fn note_anchors_on_the_lower_right_diagonal() {
    let placed = placed("chart v1 \"x\"\nnode a \"A\"\nnote \"obs\" near=a\n");
    let node = placed.node("a").expect("a");
    let note = placed.notes.first().expect("nota");
    assert!(
        note.rect.x > node.rect.right(),
        "{:?} {:?}",
        note.rect,
        node.rect
    );
    assert!(
        note.rect.y > node.rect.bottom(),
        "{:?} {:?}",
        note.rect,
        node.rect
    );
}

#[test]
fn group_box_wraps_members_and_padding() {
    let placed = placed(
        "chart v1 \"x\"\ngroup g \"G\" pad=24\nnode a \"A\" in=g\nnode b \"B\" in=g\nedge a -> b\n",
    );
    let group = placed.groups.first().expect("grupo");
    for id in ["a", "b"] {
        let node = placed.node(id).expect(id);
        assert!(group.rect.contains(node.rect.center()), "{id}");
        assert!(group.rect.x < node.rect.x && group.rect.y < node.rect.y);
    }
}

#[test]
fn edge_route_touches_both_outlines_and_carries_arrows() {
    let placed =
        placed("chart v1 \"x\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b head=arrow tail=none\n");
    let edge = placed.edges.first().expect("aresta");
    let a = placed.node("a").expect("a").rect;
    let b = placed.node("b").expect("b").rect;
    let start = edge.route.first().copied().expect("início");
    let end = edge.route.last().copied().expect("fim");
    assert!(
        (start.0 - a.right()).abs() < 0.5 || (start.1 - a.bottom()).abs() < 0.5,
        "{start:?}"
    );
    assert!(
        (end.0 - b.x).abs() < 0.5 || (end.1 - b.y).abs() < 0.5,
        "{end:?}"
    );
    assert!(edge.head);
    assert!(!edge.tail);
}

#[test]
fn self_edge_leaves_the_node_to_the_right() {
    let placed = placed("chart v1 \"x\"\nnode a \"A\"\nedge a -> a\n");
    let edge = placed.edges.first().expect("aresta");
    let node = placed.node("a").expect("a").rect;
    assert!(edge.route.iter().all(|point| point.0 >= node.right() - 0.5));
}

#[test]
fn layout_is_deterministic() {
    let source = "chart v1 \"x\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n";
    assert_eq!(placed(source), placed(source));
}

#[test]
fn canvas_includes_padding_and_grows_with_content() {
    let small = placed("chart v1 \"x\"\n");
    let big = placed(
        "chart v1 \"x\" pad=32 layout=free\nnode a \"A\" at=0,0\nnode b \"B\" at=600,400\nedge a -> b\n",
    );
    assert!(
        small.width >= 64.0 && small.height >= 64.0,
        "{}x{}",
        small.width,
        small.height
    );
    assert!(
        big.width > 600.0 && big.height > 400.0,
        "{}x{}",
        big.width,
        big.height
    );
}

/// Quantas coordenadas distintas existem, com tolerância de meio pixel.
fn distinct_count(values: &[f32]) -> usize {
    let mut seen: Vec<f32> = Vec::new();
    for value in values {
        if !seen.iter().any(|other| (other - value).abs() < 0.5) {
            seen.push(*value);
        }
    }
    seen.len()
}
