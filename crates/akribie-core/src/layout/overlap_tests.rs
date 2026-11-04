//! Invariantes de rótulo sobre os diagramas reais.
//!
//! Os quatro `.akrc` de `fixtures/reais/` são a regressão de renderização do projeto: são
//! diagramas de verdade, com rótulo de aresta em quase toda aresta e um grupo com título. O que
//! se afirma aqui vale para qualquer fonte — se a colocação de rótulo regredir, estes testes
//! apontam o fixture, a aresta e os retângulos envolvidos.

use crate::config::Config;
use crate::geometry::Rect;
use crate::layout::{Placed, layout};
use crate::model::Chart;
use crate::paint::{Scene, paint};
use crate::pipeline::check;
use crate::text::{self, Font};

const FIXTURES: [(&str, &str); 4] = [
    (
        "anatomia-modulo",
        include_str!("../../../../fixtures/reais/anatomia-modulo.akrc"),
    ),
    (
        "boot-descoberta",
        include_str!("../../../../fixtures/reais/boot-descoberta.akrc"),
    ),
    (
        "pipeline-cms-rbac",
        include_str!("../../../../fixtures/reais/pipeline-cms-rbac.akrc"),
    ),
    (
        "topologia-runtime",
        include_str!("../../../../fixtures/reais/topologia-runtime.akrc"),
    ),
];

/// Analisa, resolve, posiciona e pinta — o caminho real de `render`.
fn build(source: &str) -> (Chart, Placed, Scene, Font) {
    let font = Font::load().expect("fontes");
    let report = check(source, &Config::default(), &font);
    assert!(
        report.diagnostics.is_empty(),
        "fonte de teste precisa estar limpa: {:?}",
        report.diagnostics
    );
    let placed = layout(&report.chart, &font);
    let scene = paint(&report.chart, &placed);
    (report.chart, placed, scene, font)
}

/// Rótulos de aresta presentes no desenho, com o identificador para a mensagem.
fn edge_labels(placed: &Placed) -> Vec<(String, Rect)> {
    placed
        .edges
        .iter()
        .filter_map(|edge| {
            let rect = edge.label_rect?;
            Some((format!("{}->{}", edge.from, edge.to), rect))
        })
        .collect()
}

fn assert_clear_of_nodes(name: &str, placed: &Placed, labels: &[(String, Rect)]) {
    for (tag, rect) in labels {
        for node in &placed.nodes {
            assert!(
                !rect.intersects(node.rect),
                "{name}: rótulo `{tag}` em {rect:?} encosta no nó `{}` {:?}",
                node.id,
                node.rect
            );
        }
    }
}

fn assert_labels_do_not_touch(name: &str, labels: &[(String, Rect)]) {
    for (position, (tag, rect)) in labels.iter().enumerate() {
        for (other_tag, other) in labels.iter().skip(position.saturating_add(1)) {
            assert!(
                !rect.intersects(*other),
                "{name}: rótulo `{tag}` em {rect:?} encosta no rótulo `{other_tag}` {other:?}"
            );
        }
    }
}

#[test]
fn edge_labels_clear_nodes_and_each_other() {
    for (name, source) in FIXTURES {
        let (_, placed, _, _) = build(source);
        let labels = edge_labels(&placed);
        assert_clear_of_nodes(name, &placed, &labels);
        assert_labels_do_not_touch(name, &labels);
    }
}

#[test]
fn group_labels_fit_inside_the_group() {
    for (name, source) in FIXTURES {
        let (chart, placed, _, font) = build(source);
        let gauge = font.gauge(chart.font);
        for group in &placed.groups {
            for line in &group.label_lines {
                let width = gauge.advance_px(line, chart.fontsize) + 2.0 * text::NODE_PADDING;
                assert!(
                    width <= group.rect.w + 0.01,
                    "{name}: linha `{line}` precisa de {width} px, o grupo `{}` tem {} px",
                    group.id,
                    group.rect.w
                );
            }
        }
    }
}

#[test]
fn labels_stay_inside_the_canvas() {
    for (name, source) in FIXTURES {
        let (_, placed, scene, _) = build(source);
        for edge in &placed.edges {
            let Some(rect) = edge.label_rect else {
                continue;
            };
            let inside = rect.x >= 0.0
                && rect.y >= 0.0
                && rect.right() <= scene.width + 0.01
                && rect.bottom() <= scene.height + 0.01;
            assert!(
                inside,
                "{name}: rótulo de `{}->{}` em {rect:?} sai do canvas {}x{}",
                edge.from, edge.to, scene.width, scene.height
            );
        }
    }
}
