//! Layout: do [`Chart`] à geometria posicionada.
//!
//! O perfil é escolhido por `chart.layout`; cada perfil devolve um retângulo por nó, na ordem de
//! declaração. `finish` fecha o desenho (grupos, notas, arestas) e normaliza o canvas. Nenhuma cor
//! é decidida aqui — só geometria e rótulos quebrados.

mod finish;
#[cfg(test)]
mod fit_tests;
mod flow;
mod free;
mod grid;
mod inset;
mod labels;
mod notes;
#[cfg(test)]
mod overlap_tests;
mod radial;
#[cfg(test)]
mod tests;
mod tree;

use crate::geometry::{Point, Rect};
use crate::model::{Chart, Curve, FillStyle, Head, Layout, Shape, StrokeStyle};
use crate::span::Span;
use crate::text::{self, Font, Gauge};
use crate::theme::Token;

/// Nó posicionado (forma, rótulo quebrado e estilo; sem cor de pintura).
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedNode {
    /// Id.
    pub id: String,
    /// Caixa da forma.
    pub rect: Rect,
    /// Forma.
    pub shape: Shape,
    /// Rótulo já quebrado em linhas.
    pub label: Vec<String>,
    /// Preenchimento.
    pub fill: Token,
    /// Estilo do preenchimento.
    pub fillstyle: FillStyle,
    /// Contorno.
    pub stroke: Token,
    /// Estilo do traço.
    pub style: StrokeStyle,
    /// Espessura do contorno.
    pub width: f32,
    /// Posição no fonte.
    pub span: Span,
}

/// Aresta posicionada (rota em pontos; o SVG transforma em `path`).
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedEdge {
    /// Id de origem.
    pub from: String,
    /// Id de destino.
    pub to: String,
    /// Índice entre arestas com o mesmo par `(from, to)`.
    pub ordinal: usize,
    /// Curva declarada (`line`, `arc`, `elbow`).
    pub curve: Curve,
    /// Rótulo, quando declarado.
    pub label: Option<String>,
    /// Onde o rótulo é desenhado (meio da rota).
    pub label_at: Point,
    /// Caixa do fundo do rótulo (colisão já resolvida), quando há rótulo.
    pub label_rect: Option<Rect>,
    /// Rótulo já quebrado na largura máxima.
    pub label_lines: Vec<String>,
    /// Rota, do contorno de origem ao contorno de destino.
    pub route: Vec<Point>,
    /// Ponta com seta na chegada.
    pub head: bool,
    /// Ponta com seta na partida.
    pub tail: bool,
    /// Cor.
    pub stroke: Token,
    /// Estilo.
    pub style: StrokeStyle,
    /// Espessura.
    pub width: f32,
    /// Posição no fonte.
    pub span: Span,
}

/// Grupo posicionado (caixa envolvente dos membros + `pad`).
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedGroup {
    /// Id.
    pub id: String,
    /// Caixa.
    pub rect: Rect,
    /// Rótulo.
    pub label: String,
    /// Rótulo já quebrado na largura do grupo.
    pub label_lines: Vec<String>,
    /// Contorno.
    pub stroke: Token,
    /// Estilo do traço.
    pub style: StrokeStyle,
    /// Espessura.
    pub width: f32,
    /// Posição no fonte.
    pub span: Span,
}

/// Nota posicionada.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedNote {
    /// Caixa.
    pub rect: Rect,
    /// Texto já quebrado.
    pub text: Vec<String>,
    /// Preenchimento.
    pub fill: Token,
    /// Posição no fonte.
    pub span: Span,
}

/// Desenho posicionado, pronto para pintura.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    /// Largura do canvas, margem incluída.
    pub width: f32,
    /// Altura do canvas, margem incluída.
    pub height: f32,
    /// Grupos (atrás de tudo).
    pub groups: Vec<PlacedGroup>,
    /// Nós.
    pub nodes: Vec<PlacedNode>,
    /// Arestas.
    pub edges: Vec<PlacedEdge>,
    /// Notas.
    pub notes: Vec<PlacedNote>,
}

pub use inset::text_box;

impl Placed {
    /// Nó posicionado por id.
    pub fn node(&self, id: &str) -> Option<&PlacedNode> {
        self.nodes.iter().find(|node| node.id == id)
    }
}

/// Executa o layout completo.
pub fn layout(chart: &Chart, font: &Font) -> Placed {
    let gauge = font.gauge(chart.font);
    let sizes = node_sizes(chart, gauge);
    let labels = labels::edge_sizes(chart, gauge);
    let mut rects = match chart.layout {
        Layout::Flow => flow::run(chart, &sizes, &labels),
        Layout::Tree => tree::run(chart, &sizes),
        Layout::Mindmap => radial::run(chart, &sizes),
        Layout::Grid | Layout::Stack => grid::run(&sizes, chart.layout),
        Layout::Free => free::run(chart, &sizes),
    };
    // Pinos valem em qualquer perfil: `at` fixa o canto superior esquerdo.
    for (index, node) in chart.nodes.iter().enumerate() {
        if let (Some(at), Some(rect)) = (node.at, rects.get_mut(index)) {
            rect.x = at.0;
            rect.y = at.1;
        }
    }
    finish::assemble(chart, gauge, &sizes, &rects)
}

/// Tamanho de cada nó na ordem de `chart.nodes` (`size` declarado ou medição do rótulo).
///
/// O tamanho vem da caixa segura da forma: um losango cresce até o texto caber dentro do contorno,
/// não apenas dentro do retângulo envolvente.
fn node_sizes(chart: &Chart, gauge: &Gauge) -> Vec<(f32, f32)> {
    chart
        .nodes
        .iter()
        .map(|node| {
            node.size.unwrap_or_else(|| {
                let (text_w, text_h) = gauge.label_size(&node.label, chart.fontsize);
                let (width, height) = inset::node_box(node.shape, text_w, text_h);
                (width.max(text::MIN_NODE_W), height.max(text::MIN_NODE_H))
            })
        })
        .collect()
}

/// Rótulo quebrado para a largura útil da caixa segura do nó.
fn label_lines(
    label: &str,
    shape: Shape,
    size: (f32, f32),
    fontsize: f32,
    gauge: &Gauge,
) -> Vec<String> {
    let usable = text_box(shape, Rect::new(0.0, 0.0, size.0, size.1)).w;
    // O tamanho do nó soma `NODE_PADDING` e a caixa segura o subtrai; o arredondamento do `f32`
    // pode devolver um fio menos que a largura medida e quebrar um rótulo que cabia exatamente.
    gauge.wrap(label, fontsize, usable + 0.05)
}

/// Flags de seta a partir dos `Head` do modelo.
fn arrows(head: Head, tail: Head) -> (bool, bool) {
    (head == Head::Arrow, tail == Head::Arrow)
}
