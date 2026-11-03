//! Pintura: `Placed` + tema + fonte → [`Scene`].
//!
//! `Scene` é a última estrutura antes dos bytes: formas já em `d` (traço à mão), texto já quebrado
//! e cores já resolvidas para hex do tema. Nenhum emissor precisa saber de `Token`, `Shape` ou
//! `Curve` — só desenhar.

use crate::a11y;
use crate::geometry::Rect;
use crate::layout::Placed;
use crate::model::{Chart, StrokeStyle};
use crate::sketch::{Ink, seed_for};
use crate::text::{self, Font};

/// Raio do canto de `shape=rounded`.
pub const CORNER_RADIUS: f32 = 12.0;
/// Comprimento da cabeça de seta.
pub const HEAD_LEN: f32 = 12.0;
/// Respiro do fundo do rótulo de aresta: separa o texto da linha em, no mínimo, 6 px.
pub const LABEL_PAD: f32 = 6.0;

/// Alinhamento horizontal do texto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// Centrado (rótulos de nó e aresta).
    Middle,
    /// À esquerda (notas e rótulos de grupo).
    Start,
}

/// Um pedaço de texto posicionado.
#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    /// Conteúdo.
    pub text: String,
    /// Coordenada do ponto de ancoragem.
    pub x: f32,
    /// Linha de base.
    pub y: f32,
    /// Tamanho da fonte, em pixels.
    pub size: f32,
    /// Cor (hex do tema).
    pub color: &'static str,
    /// Alinhamento.
    pub anchor: Anchor,
}

/// Traço desenhado.
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    /// Caminho.
    pub d: String,
    /// Cor.
    pub color: &'static str,
    /// Espessura.
    pub width: f32,
    /// `stroke-dasharray`, quando tracejado/pontilhado.
    pub dash: Option<&'static str>,
}

/// Preenchimento (sólido ou hachura/zigzag dentro do `clip` do elemento).
#[derive(Debug, Clone, PartialEq)]
pub struct Fill {
    /// Caminho.
    pub d: String,
    /// Cor.
    pub color: &'static str,
    /// `true` = preencher; `false` = traçar o caminho (hachura).
    pub solid: bool,
}

/// Nó desenhado.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeDraw {
    /// Id.
    pub id: String,
    /// Rótulo acessível.
    pub aria_label: Option<String>,
    /// Caminho de recorte (quando a hachura precisa ficar dentro da forma).
    pub clip: Option<String>,
    /// Preenchimento.
    pub fill: Option<Fill>,
    /// Contorno.
    pub stroke: Option<Stroke>,
    /// Texto.
    pub text: Vec<TextRun>,
}

/// Rótulo de aresta com fundo.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelDraw {
    /// Texto.
    pub text: TextRun,
    /// Fundo (cor do canvas).
    pub rect: Rect,
}

/// Aresta desenhada.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeDraw {
    /// Identificador estável (`from->to#ordinal`).
    pub id: String,
    /// Rótulo acessível.
    pub aria_label: Option<String>,
    /// Rota.
    pub route: Stroke,
    /// Seta na chegada.
    pub head: Option<Fill>,
    /// Seta na partida.
    pub tail: Option<Fill>,
    /// Rótulo.
    pub label: Option<LabelDraw>,
}

/// Grupo desenhado.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupDraw {
    /// Id.
    pub id: String,
    /// Rótulo acessível.
    pub aria_label: Option<String>,
    /// Contorno.
    pub stroke: Stroke,
    /// Rótulo.
    pub label: Option<TextRun>,
}

/// Nota desenhada.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteDraw {
    /// Identificador estável.
    pub id: String,
    /// Texto (já como rótulo acessível).
    pub aria_label: Option<String>,
    /// Preenchimento.
    pub fill: Fill,
    /// Contorno.
    pub stroke: Stroke,
    /// Texto.
    pub text: Vec<TextRun>,
}

/// Cena pronta para emissão.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    /// Largura do canvas.
    pub width: f32,
    /// Altura do canvas.
    pub height: f32,
    /// Título do documento.
    pub title: Option<String>,
    /// Idioma.
    pub lang: String,
    /// Família tipográfica a emitir no SVG.
    pub font_stack: &'static str,
    /// Descrição acessível.
    pub desc: String,
    /// Grupos.
    pub groups: Vec<GroupDraw>,
    /// Arestas.
    pub edges: Vec<EdgeDraw>,
    /// Nós.
    pub nodes: Vec<NodeDraw>,
    /// Notas.
    pub notes: Vec<NoteDraw>,
}

/// Pinta o desenho com o tema e o tamanho de fonte do documento.
pub fn paint(chart: &Chart, placed: &Placed, font: &Font) -> Scene {
    let theme = chart.theme;
    let gauge = font.gauge(chart.font);
    let finish = |seed: u32| {
        if chart.sketch {
            Ink::Rough(seed)
        } else {
            Ink::Exact
        }
    };
    Scene {
        width: placed.width,
        height: placed.height,
        title: chart.title.clone(),
        lang: chart.lang.clone(),
        font_stack: text::stack(chart.font),
        desc: a11y::describe(chart),
        groups: placed
            .groups
            .iter()
            .map(|group| paint_group(group, theme, chart.fontsize, finish(seed_for(&group.id))))
            .collect(),
        edges: placed
            .edges
            .iter()
            .map(|edge| {
                let seed = seed_for(&edge.from) ^ seed_for(&edge.to);
                paint_edge(edge, theme, chart.fontsize, gauge, finish(seed))
            })
            .collect(),
        nodes: placed
            .nodes
            .iter()
            .map(|node| paint_node(node, theme, chart.fontsize, finish(seed_for(&node.id))))
            .collect(),
        notes: placed
            .notes
            .iter()
            .map(|note| {
                paint_note(
                    note,
                    theme,
                    chart.fontsize,
                    finish(seed_for(&format!("note:{}", note.span.line))),
                )
            })
            .collect(),
    }
}

mod edge;
mod group;
mod node;
#[cfg(test)]
mod tests;

use edge::paint_edge;
use group::{paint_group, paint_note};
use node::paint_node;

fn text_runs(
    label: &[String],
    rect: Rect,
    fontsize: f32,
    color: &'static str,
    anchor: Anchor,
) -> Vec<TextRun> {
    let line_height = fontsize * text::LINE_HEIGHT;
    let count = label.len();
    let first =
        rect.center().1 - (f32::from(u16::try_from(count).unwrap_or(1)) - 1.0) * line_height * 0.5;
    label
        .iter()
        .enumerate()
        .map(|(index, line)| TextRun {
            text: line.clone(),
            x: rect.center().0,
            y: first + line_height * f32::from(u16::try_from(index).unwrap_or(0)),
            size: fontsize,
            color,
            anchor,
        })
        .collect()
}

fn dash_of(style: StrokeStyle) -> Option<&'static str> {
    match style {
        StrokeStyle::Solid => None,
        StrokeStyle::Dashed => Some("8 8"),
        StrokeStyle::Dotted => Some("1.5 6"),
    }
}

fn aria_of(labels: &[String]) -> Option<String> {
    let text = labels.join(" ");
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
