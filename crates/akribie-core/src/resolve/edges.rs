//! Arestas e nós implícitos: referências que criam nós em vez de falhar.

use super::rules::missing_node;
use crate::diag::{Diagnostic, E005, W001};
use crate::model::{Curve, Edge, FillStyle, Head, Node, Shape, StrokeStyle};
use crate::parse::Document;
use crate::span::Span;
use crate::theme::Token;

use super::{IdKind, State, pick, token, width};

impl State {
    pub(super) fn declare_edges(&mut self, document: &Document) {
        for stmt in &document.edges {
            let from_ok = self.ensure_node(&stmt.from, stmt.span);
            let to_ok = self.ensure_node(&stmt.to, stmt.span);
            if !from_ok || !to_ok {
                continue;
            }
            let key = (stmt.from.clone(), stmt.to.clone());
            let ordinal = self.ordinals.entry(key).or_insert(0);
            let edge = Edge {
                from: stmt.from.clone(),
                to: stmt.to.clone(),
                label: stmt.label.clone(),
                curve: pick(&stmt.options, "curve", Curve::Line, Curve::from_name),
                head: pick(&stmt.options, "head", Head::Arrow, Head::from_name),
                tail: pick(&stmt.options, "tail", Head::None, Head::from_name),
                style: pick(
                    &stmt.options,
                    "style",
                    StrokeStyle::Solid,
                    StrokeStyle::from_name,
                ),
                stroke: token(&stmt.options, "stroke", Token::Ink),
                width: width(&stmt.options),
                ordinal: *ordinal,
                span: stmt.span,
            };
            *ordinal += 1;
            self.chart.edges.push(edge);
        }
    }

    /// Garante que o id de uma ponta de aresta é um nó; cria o implícito quando não existe.
    fn ensure_node(&mut self, id: &str, span: Span) -> bool {
        if self.chart.node(id).is_some() {
            return true;
        }
        match self.ids.get(id) {
            Some(IdKind::Group) => {
                self.diagnostics.push(
                    Diagnostic::new(E005, span, format!("`{id}` é um grupo; arestas ligam nós"))
                        .with_hint("ligue nós, ou crie um `node` que represente o grupo"),
                );
                false
            }
            Some(IdKind::Node) => true,
            None => {
                self.ids.insert(id.to_string(), IdKind::Node);
                self.diagnostics.push(
                    Diagnostic::new(
                        W001,
                        span,
                        format!("`{id}` não foi declarado; criado como nó implícito"),
                    )
                    .with_hint("declare o nó para controlar forma, cor e rótulo"),
                );
                self.chart.nodes.push(Node {
                    id: id.to_string(),
                    label: id.to_string(),
                    declared_label: None,
                    shape: Shape::Rounded,
                    fill: Token::None,
                    fillstyle: FillStyle::Hachure,
                    stroke: Token::Ink,
                    style: StrokeStyle::Solid,
                    width: 2.0,
                    groups: Vec::new(),
                    at: None,
                    size: None,
                    span,
                    implicit: true,
                });
                true
            }
        }
    }

    /// `true` quando `id` é um nó declarado; senão reporta `E005`.
    pub(super) fn checked_node(&mut self, id: &str, span: Span) -> bool {
        if self.ids.get(id) == Some(&IdKind::Node) {
            return true;
        }
        self.diagnostics
            .push(missing_node(id, span, "`near=` aponta para um nó"));
        false
    }
}
