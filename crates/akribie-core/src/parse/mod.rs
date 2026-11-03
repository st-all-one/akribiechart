//! Parser do `.akrc`: linhas → [`Document`] + diagnósticos.
//!
//! Recuperação é **por linha**: uma linha inválida produz um diagnóstico e não invalida as outras.
//! É o que permite que um LLM receba todas as correções de uma vez, em vez de descobri-las uma a
//! uma a cada reenvio.

mod opts;
mod stmt;

use crate::diag::{Diagnostic, E001, E007, E008, E009};
use crate::lex::{self, Token};
use crate::span::Span;
use crate::syntax::{ChartStmt, EdgeStmt, GroupStmt, NodeStmt, NoteStmt, StmtKind};

/// Documento sintático: a estrutura do arquivo, sem semântica aplicada.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Document {
    /// Statement `chart` (o único obrigatório).
    pub chart: Option<ChartStmt>,
    /// Statements `group`, na ordem de declaração.
    pub groups: Vec<GroupStmt>,
    /// Statements `node`, na ordem de declaração.
    pub nodes: Vec<NodeStmt>,
    /// Statements `edge`, na ordem de declaração.
    pub edges: Vec<EdgeStmt>,
    /// Statements `note`, na ordem de declaração.
    pub notes: Vec<NoteStmt>,
    /// Comentários sem statement seguinte (ficam no fim do arquivo em `fmt`).
    pub trailing_comments: Vec<String>,
}

/// Analisa o fonte inteiro. Nunca falha: irregularidades viram diagnósticos.
pub fn parse(src: &str) -> (Document, Vec<Diagnostic>) {
    let mut document = Document::default();
    let mut diagnostics = Vec::new();
    let mut pending_comments: Vec<String> = Vec::new();
    let mut chart_seen = false;
    let mut chart_gap_reported = false;

    for (index, raw) in src.split('\n').enumerate() {
        let line_number = u32::try_from(index.saturating_add(1)).unwrap_or(u32::MAX);
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let content = line.trim_start_matches(' ');
        if content.is_empty() {
            continue;
        }
        if content.starts_with('#') {
            pending_comments.push(content.trim_end().to_string());
            continue;
        }
        if content.contains('\t') {
            diagnostics.push(tab_error(line_number));
            continue;
        }
        let tokens = match lex::lex(line, Span::line_start(line_number)) {
            Ok(tokens) => tokens,
            Err(diagnostic) => {
                diagnostics.push(diagnostic);
                continue;
            }
        };
        let Some(keyword) = tokens.first() else {
            continue;
        };
        let Some(kind) = keyword_kind(&keyword.text) else {
            diagnostics.push(
                Diagnostic::new(
                    E001,
                    keyword.span,
                    format!("afirmação desconhecida `{}`", keyword.text),
                )
                .with_hint("uma afirmação começa com: chart, group, node, edge ou note"),
            );
            continue;
        };
        if !chart_seen && kind != StmtKind::Chart && !chart_gap_reported {
            chart_gap_reported = true;
            diagnostics.push(
                Diagnostic::new(
                    E008,
                    keyword.span,
                    format!(
                        "a primeira afirmação é `{}`, mas precisa ser `chart`",
                        keyword.text
                    ),
                )
                .with_hint("coloque `chart v1 \"Título\"` na primeira linha não vazia"),
            );
        }

        let comments = std::mem::take(&mut pending_comments);
        // Cada parser de statement consome a própria palavra-chave como `head`.
        let mut cursor = Cursor::new(&tokens);
        if kind == StmtKind::Chart {
            if chart_seen {
                diagnostics.push(
                    Diagnostic::new(E008, keyword.span, "declaração `chart` duplicada").with_hint(
                        "o arquivo tem exatamente uma linha `chart`, e ela é a primeira",
                    ),
                );
                continue;
            }
            chart_seen = true;
            document.chart = stmt::chart(&mut cursor, comments, &mut diagnostics);
            continue;
        }
        match kind {
            StmtKind::Chart => {}
            StmtKind::Group => {
                if let Some(item) = stmt::group(&mut cursor, comments, &mut diagnostics) {
                    document.groups.push(item);
                }
            }
            StmtKind::Node => {
                if let Some(item) = stmt::node(&mut cursor, comments, &mut diagnostics) {
                    document.nodes.push(item);
                }
            }
            StmtKind::Edge => {
                if let Some(item) = stmt::edge(&mut cursor, comments, &mut diagnostics) {
                    document.edges.push(item);
                }
            }
            StmtKind::Note => {
                if let Some(item) = stmt::note(&mut cursor, comments, &mut diagnostics) {
                    document.notes.push(item);
                }
            }
        }
    }

    document.trailing_comments = pending_comments;
    diagnostics.sort_by(|a, b| {
        (a.span.line, a.span.col, a.code.code).cmp(&(b.span.line, b.span.col, b.code.code))
    });
    (document, diagnostics)
}

fn tab_error(line_number: u32) -> Diagnostic {
    Diagnostic::new(E007, Span::line_start(line_number), "a linha contém tab")
        .with_hint("substitua o tab por um ou mais espaços")
}

/// Erro de linha malformada com posição.
pub(crate) fn malformed(
    span: Span,
    message: impl Into<String>,
    hint: impl Into<String>,
) -> Diagnostic {
    Diagnostic::new(E007, span, message).with_hint(hint)
}

/// Erro de versão não suportada.
pub(crate) fn unsupported_version(span: Span, version: &str) -> Diagnostic {
    Diagnostic::new(E009, span, format!("versão `{version}` não suportada"))
        .with_hint("a única versão aceita na v1 do formato é `v1`")
}

fn keyword_kind(text: &str) -> Option<StmtKind> {
    match text {
        "chart" => Some(StmtKind::Chart),
        "group" => Some(StmtKind::Group),
        "node" => Some(StmtKind::Node),
        "edge" => Some(StmtKind::Edge),
        "note" => Some(StmtKind::Note),
        _ => None,
    }
}

/// Cursor sobre os tokens de uma linha.
pub(crate) struct Cursor<'t> {
    tokens: &'t [Token],
    at: usize,
}

impl<'t> Cursor<'t> {
    pub(crate) fn new(tokens: &'t [Token]) -> Self {
        Self { tokens, at: 0 }
    }

    pub(crate) fn next(&mut self) -> Option<&'t Token> {
        let token = self.tokens.get(self.at);
        if token.is_some() {
            self.at = self.at.saturating_add(1);
        }
        token
    }

    pub(crate) fn peek(&self) -> Option<&'t Token> {
        self.tokens.get(self.at)
    }
}
