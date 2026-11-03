//! Modelo resolvido: o que o desenho precisa saber, sem sintaxe.
//!
//! `resolve` transforma um [`crate::parse::Document`] neste modelo; layout, pintura e emissão só
//! conhecem [`Chart`] e seus filhos. Nenhuma estrutura guarda opções cruas: tipos fechados em
//! enums, tokens de cor e geometria declarada (`at`, `size`).

mod enums;
mod family;

pub use enums::{Curve, Direction, FillStyle, Head, Layout, Shape, StrokeStyle};
pub use family::FontFamily;

use crate::span::Span;
use crate::theme::{Theme, Token};

/// Documento resolvido (o que o pipeline consome).
#[derive(Debug, Clone, PartialEq)]
pub struct Chart {
    /// Título; ausente ⇒ SVG sem `<title>`/`aria-label`.
    pub title: Option<String>,
    /// Perfil de layout.
    pub layout: Layout,
    /// Direção do fluxo.
    pub direction: Direction,
    /// Tema.
    pub theme: &'static Theme,
    /// Tamanho de fonte (14–96).
    pub fontsize: f32,
    /// Margem externa do canvas.
    pub pad: f32,
    /// Idioma (BCP-47) emitido como `lang`.
    pub lang: String,
    /// Traço à mão (`true`) ou traço técnico (`false`).
    pub sketch: bool,
    /// Família tipográfica das medições e do desenho.
    pub font: FontFamily,
    /// Linhagem (`doc=path#anchor`), metadado puro.
    pub doc: Option<String>,
    /// Grupos, na ordem de declaração.
    pub groups: Vec<Group>,
    /// Nós, na ordem de declaração (implícitos ao final).
    pub nodes: Vec<Node>,
    /// Arestas, na ordem de declaração.
    pub edges: Vec<Edge>,
    /// Notas, na ordem de declaração.
    pub notes: Vec<Note>,
}

/// Grupo: retângulo que envolve nós.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    /// Id.
    pub id: String,
    /// Rótulo (o id quando ausente).
    pub label: String,
    /// Rótulo declarado explicitamente.
    pub declared_label: Option<String>,
    /// Preenchimento.
    pub fill: Token,
    /// Contorno.
    pub stroke: Token,
    /// Estilo do traço.
    pub style: StrokeStyle,
    /// Espessura do contorno.
    pub width: f32,
    /// Padding interno.
    pub pad: f32,
    /// Grupos que contêm este grupo.
    pub groups: Vec<String>,
    /// Posição no fonte.
    pub span: Span,
}

/// Nó: elemento desenhado.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Id.
    pub id: String,
    /// Rótulo (o id quando ausente).
    pub label: String,
    /// Rótulo declarado explicitamente.
    pub declared_label: Option<String>,
    /// Forma.
    pub shape: Shape,
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
    /// Grupos que contêm este nó.
    pub groups: Vec<String>,
    /// Posição declarada (pino).
    pub at: Option<(f32, f32)>,
    /// Tamanho declarado (sobrepõe a medição).
    pub size: Option<(f32, f32)>,
    /// Posição no fonte.
    pub span: Span,
    /// `true` quando criado por uma aresta (`W001`).
    pub implicit: bool,
}

/// Aresta dirigida.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    /// Id de origem.
    pub from: String,
    /// Id de destino.
    pub to: String,
    /// Rótulo, quando declarado.
    pub label: Option<String>,
    /// Curva.
    pub curve: Curve,
    /// Ponta final.
    pub head: Head,
    /// Ponta inicial.
    pub tail: Head,
    /// Estilo do traço.
    pub style: StrokeStyle,
    /// Cor do traço.
    pub stroke: Token,
    /// Espessura.
    pub width: f32,
    /// Índice entre arestas com o mesmo par `(from, to)`.
    pub ordinal: usize,
    /// Posição no fonte.
    pub span: Span,
}

/// Nota: anotação em texto livre.
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    /// Texto.
    pub text: String,
    /// Preenchimento.
    pub fill: Token,
    /// Posição declarada.
    pub at: Option<(f32, f32)>,
    /// Tamanho declarado.
    pub size: Option<(f32, f32)>,
    /// Nó ao qual a nota se ancora.
    pub near: Option<String>,
    /// Posição no fonte.
    pub span: Span,
}

impl Chart {
    /// Nó por id.
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|node| node.id == id)
    }

    /// Grupo por id.
    pub fn group(&self, id: &str) -> Option<&Group> {
        self.groups.iter().find(|group| group.id == id)
    }

    /// Grau (arestas incidentes) de um nó.
    pub fn degree(&self, id: &str) -> usize {
        self.edges
            .iter()
            .filter(|edge| edge.from == id || edge.to == id)
            .count()
    }

    /// Rótulo de um id (o próprio id quando não existir).
    pub fn label_of<'a>(&'a self, id: &'a str) -> &'a str {
        self.node(id).map_or(id, |node| node.label.as_str())
    }
}
