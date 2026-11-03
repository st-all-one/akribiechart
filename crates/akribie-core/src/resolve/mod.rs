//! Resolução semântica: [`Document`] + [`Config`] → [`Chart`].
//!
//! O parser garante a forma; aqui aplicamos o significado: ids únicos, referências existentes,
//! implícitos de aresta, posições obrigatórias em `layout=free`. Nada de I/O, nada de geometria —
//! só o que o desenho precisa saber, com diagnósticos posicionados.

mod edges;
mod rules;
#[cfg(test)]
mod tests;

use indexmap::IndexMap;

use crate::config::Config;
use crate::diag::{Diagnostic, E004};
use crate::model::{
    Chart, Direction, FillStyle, FontFamily, Group, Layout, Node, Note, Shape, StrokeStyle,
};
use crate::parse::Document;
use crate::span::Span;
use crate::syntax::{self, GroupStmt, NodeStmt, Options};
use crate::theme::{self, Token};

use rules::{missing_group, missing_position};

/// Classe de um id declarado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdKind {
    Group,
    Node,
}

/// Resolve o documento contra a configuração do projeto.
pub fn resolve(document: &Document, config: &Config) -> (Chart, Vec<Diagnostic>) {
    let empty = Options::new();
    let options = document
        .chart
        .as_ref()
        .map_or(&empty, |chart| &chart.options);
    let chart = Chart {
        title: document.chart.as_ref().and_then(|stmt| stmt.title.clone()),
        layout: pick(options, "layout", config.layout, Layout::from_name),
        direction: pick(options, "direction", config.direction, Direction::from_name),
        theme: theme::theme(syntax::text(options, "theme").unwrap_or(config.theme.name))
            .unwrap_or(config.theme),
        fontsize: syntax::number(options, "fontsize").unwrap_or(config.fontsize),
        pad: syntax::number(options, "pad").unwrap_or(config.pad),
        lang: syntax::text(options, "lang")
            .unwrap_or(&config.lang)
            .to_string(),
        sketch: pick(options, "sketch", config.sketch, |name| match name {
            "on" => Some(true),
            "off" => Some(false),
            _ => None,
        }),
        font: pick(options, "font", config.font, FontFamily::from_name),
        doc: syntax::text(options, "doc").map(str::to_string),
        groups: Vec::new(),
        nodes: Vec::new(),
        edges: Vec::new(),
        notes: Vec::new(),
    };
    let mut state = State {
        chart,
        diagnostics: Vec::new(),
        ids: IndexMap::new(),
        ordinals: IndexMap::new(),
    };
    let group_ok: Vec<bool> = document
        .groups
        .iter()
        .map(|stmt| state.declare(&stmt.id, IdKind::Group, stmt.span))
        .collect();
    let node_ok: Vec<bool> = document
        .nodes
        .iter()
        .map(|stmt| state.declare(&stmt.id, IdKind::Node, stmt.span))
        .collect();
    for (stmt, ok) in document.groups.iter().zip(group_ok) {
        if ok {
            state.build_group(stmt);
        }
    }
    for (stmt, ok) in document.nodes.iter().zip(node_ok) {
        if ok {
            state.build_node(stmt);
        }
    }
    state.declare_notes(document);
    state.declare_edges(document);
    state.check_positions();
    (state.chart, state.diagnostics)
}

/// Enum fechado a partir de uma opção textual, com default.
pub(super) fn pick<T: Copy>(
    options: &Options,
    key: &str,
    default: T,
    parse: fn(&str) -> Option<T>,
) -> T {
    syntax::text(options, key)
        .and_then(parse)
        .unwrap_or(default)
}

pub(super) fn token(options: &Options, key: &str, default: Token) -> Token {
    syntax::text(options, key)
        .and_then(Token::from_name)
        .unwrap_or(default)
}

pub(super) fn width(options: &Options) -> f32 {
    syntax::number(options, "width").unwrap_or(2.0)
}

struct State {
    chart: Chart,
    diagnostics: Vec<Diagnostic>,
    ids: IndexMap<String, IdKind>,
    ordinals: IndexMap<(String, String), usize>,
}

impl State {
    fn declare(&mut self, id: &str, kind: IdKind, span: Span) -> bool {
        if self.ids.contains_key(id) {
            self.diagnostics.push(
                Diagnostic::new(E004, span, format!("id `{id}` já declarado"))
                    .with_hint("ids são únicos entre `group` e `node`; renomeie um dos dois"),
            );
            return false;
        }
        self.ids.insert(id.to_string(), kind);
        true
    }

    fn build_group(&mut self, stmt: &GroupStmt) {
        let groups = self.groups_of(&stmt.options, stmt.span);
        self.chart.groups.push(Group {
            id: stmt.id.clone(),
            label: stmt.label.clone().unwrap_or_else(|| stmt.id.clone()),
            declared_label: stmt.label.clone(),
            fill: token(&stmt.options, "fill", Token::None),
            stroke: token(&stmt.options, "stroke", Token::Ink),
            style: pick(
                &stmt.options,
                "style",
                StrokeStyle::Solid,
                StrokeStyle::from_name,
            ),
            width: width(&stmt.options),
            pad: syntax::number(&stmt.options, "pad").unwrap_or(24.0),
            groups,
            span: stmt.span,
        });
    }

    fn build_node(&mut self, stmt: &NodeStmt) {
        let groups = self.groups_of(&stmt.options, stmt.span);
        self.chart.nodes.push(Node {
            id: stmt.id.clone(),
            label: stmt.label.clone().unwrap_or_else(|| stmt.id.clone()),
            declared_label: stmt.label.clone(),
            shape: pick(&stmt.options, "shape", Shape::Rounded, Shape::from_name),
            fill: token(&stmt.options, "fill", Token::None),
            fillstyle: pick(
                &stmt.options,
                "fillstyle",
                FillStyle::Hachure,
                FillStyle::from_name,
            ),
            stroke: token(&stmt.options, "stroke", Token::Ink),
            style: pick(
                &stmt.options,
                "style",
                StrokeStyle::Solid,
                StrokeStyle::from_name,
            ),
            width: width(&stmt.options),
            groups,
            at: syntax::pair(&stmt.options, "at"),
            size: syntax::pair(&stmt.options, "size"),
            span: stmt.span,
            implicit: false,
        });
    }

    fn declare_notes(&mut self, document: &Document) {
        for stmt in &document.notes {
            let near = syntax::text(&stmt.options, "near")
                .filter(|id| self.checked_node(id, stmt.span))
                .map(str::to_string);
            self.chart.notes.push(Note {
                text: stmt.text.clone(),
                fill: token(&stmt.options, "fill", Token::Amber),
                at: syntax::pair(&stmt.options, "at"),
                size: syntax::pair(&stmt.options, "size"),
                near,
                span: stmt.span,
            });
        }
    }

    fn groups_of(&mut self, options: &Options, span: Span) -> Vec<String> {
        let members = syntax::ids(options, "in")
            .map(<[String]>::to_vec)
            .unwrap_or_default();
        let mut valid = Vec::new();
        for parent in members {
            if self.ids.get(&parent) == Some(&IdKind::Group) {
                valid.push(parent);
            } else {
                self.diagnostics.push(missing_group(&parent, span));
            }
        }
        valid
    }

    fn check_positions(&mut self) {
        if !self.chart.layout.requires_position() {
            return;
        }
        for node in &self.chart.nodes {
            if node.at.is_none() {
                self.diagnostics.push(missing_position(&node.id, node.span));
            }
        }
        for note in &self.chart.notes {
            if note.at.is_none() {
                self.diagnostics.push(missing_position("nota", note.span));
            }
        }
    }
}
