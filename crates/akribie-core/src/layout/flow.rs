//! `layout=flow`: DAG em camadas via dagre (`dugong`), com rota própria.
//!
//! O dagre decide ranks, ordem e coordenadas dos nós; **a rota da aresta é nossa** (ver
//! `geometry`), porque a curva é da linguagem visual do akribie, não do algoritmo.

use dugong::graphlib::{Graph, GraphOptions};
use dugong::{EdgeLabel, GraphLabel, NodeLabel, NoopWorkControl, RankDir, layout_controlled};

use crate::geometry::Rect;
use crate::model::{Chart, Direction};

use super::grid;

/// Folga horizontal entre nós irmãos.
const NODE_SEP: f64 = 48.0;
/// Folga entre camadas: precisa caber o rótulo de aresta (texto de 20 px).
const RANK_SEP: f64 = 96.0;

pub(super) fn run(chart: &Chart, sizes: &[(f32, f32)], labels: &[(f32, f32)]) -> Vec<Rect> {
    let mut graph: Graph<NodeLabel, EdgeLabel, GraphLabel> = Graph::new(GraphOptions {
        multigraph: true,
        ..GraphOptions::default()
    });
    graph.set_graph(GraphLabel {
        rankdir: rank_dir(chart.direction),
        nodesep: NODE_SEP,
        ranksep: RANK_SEP,
        ..GraphLabel::default()
    });
    for (node, (width, height)) in chart.nodes.iter().zip(sizes) {
        graph.set_node(
            node.id.as_str(),
            NodeLabel {
                width: f64::from(*width),
                height: f64::from(*height),
                ..NodeLabel::default()
            },
        );
    }
    for (index, edge) in chart.edges.iter().enumerate() {
        // A caixa do rótulo entra como nó dummy no dagre, que então separa os ranks na medida
        // do texto: sem isso o rótulo nasce em cima do nó da camada seguinte.
        let (width, height) = labels.get(index).copied().unwrap_or((0.0, 0.0));
        graph.set_edge_named(
            edge.from.as_str(),
            edge.to.as_str(),
            Some(format!("e{index}")),
            Some(EdgeLabel {
                width: f64::from(width),
                height: f64::from(height),
                ..EdgeLabel::default()
            }),
        );
    }
    if layout_controlled(&mut graph, &mut NoopWorkControl).is_err() {
        return grid::stack(sizes);
    }
    chart
        .nodes
        .iter()
        .zip(sizes)
        .map(|(node, (width, height))| {
            let label = graph.node(node.id.as_str());
            let center_x = label.and_then(|value| value.x).unwrap_or(0.0);
            let center_y = label.and_then(|value| value.y).unwrap_or(0.0);
            Rect::new(
                to_f32(center_x) - width * 0.5,
                to_f32(center_y) - height * 0.5,
                *width,
                *height,
            )
        })
        .collect()
}

const fn rank_dir(direction: Direction) -> RankDir {
    match direction {
        Direction::Lr => RankDir::LR,
        Direction::Tb => RankDir::TB,
        Direction::Bt => RankDir::BT,
        Direction::Rl => RankDir::RL,
    }
}

/// `dugong` expõe `f64`; o canvas é `f32` e as coordenadas cabem com folga.
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "conversão de interop: dagre entrega f64, o canvas é f32"
)]
fn to_f32(value: f64) -> f32 {
    value as f32
}
