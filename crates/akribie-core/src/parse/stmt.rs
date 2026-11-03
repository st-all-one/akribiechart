//! Statements individuais: cabeça da linha e posições.

use super::{Cursor, malformed, unsupported_version};
use crate::diag::Diagnostic;
use crate::lex::{TokKind, Token};
use crate::syntax::{self, ChartStmt, EdgeStmt, GroupStmt, NodeStmt, NoteStmt, StmtKind};

pub(super) fn chart(
    cursor: &mut Cursor<'_>,
    comments: Vec<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ChartStmt> {
    let head = cursor.next()?;
    let Some(version) = cursor.next() else {
        diagnostics.push(malformed(
            head.span,
            "`chart` sem versão",
            "use `chart v1 \"Título\"`",
        ));
        return None;
    };
    if version.kind != TokKind::Version || version.text != "v1" {
        diagnostics.push(unsupported_version(version.span, &version.text));
    }
    let title = take_label(cursor);
    let options = super::opts::parse(cursor, StmtKind::Chart, diagnostics);
    Some(ChartStmt {
        title,
        options,
        comments,
        span: head.span,
    })
}

pub(super) fn group(
    cursor: &mut Cursor<'_>,
    comments: Vec<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<GroupStmt> {
    let head = cursor.next()?;
    let id = take_id(cursor, "group", diagnostics)?;
    let label = take_label(cursor);
    let options = super::opts::parse(cursor, StmtKind::Group, diagnostics);
    Some(GroupStmt {
        id,
        label,
        options,
        comments,
        span: head.span,
    })
}

pub(super) fn node(
    cursor: &mut Cursor<'_>,
    comments: Vec<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<NodeStmt> {
    let head = cursor.next()?;
    let id = take_id(cursor, "node", diagnostics)?;
    let label = take_label(cursor);
    let options = super::opts::parse(cursor, StmtKind::Node, diagnostics);
    Some(NodeStmt {
        id,
        label,
        options,
        comments,
        span: head.span,
    })
}

pub(super) fn edge(
    cursor: &mut Cursor<'_>,
    comments: Vec<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<EdgeStmt> {
    let head = cursor.next()?;
    let from = take_id(cursor, "edge", diagnostics)?;
    let Some(arrow) = cursor.next() else {
        diagnostics.push(malformed(
            head.span,
            "aresta sem `->`",
            "declare `edge origem -> destino \"Rótulo\"`",
        ));
        return None;
    };
    if arrow.kind != TokKind::Arrow {
        diagnostics.push(malformed(
            arrow.span,
            format!("esperava `->`, obteve `{}`", arrow.text),
            "a seta liga dois ids: `edge origem -> destino`",
        ));
        return None;
    }
    let Some(to) = cursor.next() else {
        diagnostics.push(malformed(
            arrow.span,
            "aresta sem destino",
            "declare o id de destino depois de `->`",
        ));
        return None;
    };
    if !is_id_token(to) {
        diagnostics.push(malformed(
            to.span,
            format!("`{}` não é um id válido", to.text),
            "o destino de uma aresta é um `node` declarado ou um id implícito",
        ));
        return None;
    }
    let label = take_label(cursor);
    let options = super::opts::parse(cursor, StmtKind::Edge, diagnostics);
    Some(EdgeStmt {
        from,
        to: to.text.clone(),
        label,
        options,
        comments,
        span: head.span,
    })
}

pub(super) fn note(
    cursor: &mut Cursor<'_>,
    comments: Vec<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<NoteStmt> {
    let head = cursor.next()?;
    let Some(text) = cursor.next() else {
        diagnostics.push(malformed(
            head.span,
            "`note` sem texto",
            "use `note \"Texto da nota\"`",
        ));
        return None;
    };
    if text.kind != TokKind::Str {
        diagnostics.push(malformed(
            text.span,
            format!("`{}` não é uma string", text.text),
            "o texto da nota vai entre aspas: `note \"Texto\"`",
        ));
        return None;
    }
    let options = super::opts::parse(cursor, StmtKind::Note, diagnostics);
    Some(NoteStmt {
        text: text.text.clone(),
        options,
        comments,
        span: head.span,
    })
}

fn is_id_token(token: &Token) -> bool {
    matches!(token.kind, TokKind::Ident | TokKind::Version) && syntax::is_ident(&token.text)
}

fn take_id(
    cursor: &mut Cursor<'_>,
    statement: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    let missing_span = cursor.peek().map(|token| token.span);
    let Some(token) = cursor.next() else {
        if let Some(span) = missing_span {
            diagnostics.push(malformed(
                span,
                format!("`{statement}` sem id"),
                "use `IDENT` depois da palavra-chave, como `node api`",
            ));
        }
        return None;
    };
    if !is_id_token(token) {
        diagnostics.push(malformed(
            token.span,
            format!("`{}` não é um id válido", token.text),
            "ids começam com letra e usam letras, dígitos, `_` ou `-` (máx. 64)",
        ));
        return None;
    }
    Some(token.text.clone())
}

fn take_label(cursor: &mut Cursor<'_>) -> Option<String> {
    if cursor
        .peek()
        .is_some_and(|token| token.kind == TokKind::Str)
    {
        cursor.next().map(|token| token.text.clone())
    } else {
        None
    }
}
