#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "teste: a asserção precisa falhar alto e apontar a linha"
)]

//! Contrato do léxico: classificação de token, strings e separador.

use akribie_core::lex::{TokKind, lex};
use akribie_core::span::Span;

fn kinds(line: &str) -> Vec<TokKind> {
    lex(line, Span::line_start(1))
        .map(|tokens| tokens.into_iter().map(|token| token.kind).collect())
        .unwrap_or_default()
}

fn texts(line: &str) -> Vec<String> {
    lex(line, Span::line_start(1))
        .map(|tokens| tokens.into_iter().map(|token| token.text).collect())
        .unwrap_or_default()
}

#[test]
fn classifies_statement_head() {
    assert_eq!(
        kinds("node api \"API Gateway\""),
        vec![TokKind::Ident, TokKind::Ident, TokKind::Str]
    );
    assert_eq!(texts("chart v1 \"x\""), vec!["chart", "v1", "x"]);
    assert_eq!(kinds("chart v1"), vec![TokKind::Ident, TokKind::Version]);
}

#[test]
fn arrow_is_one_token() {
    assert_eq!(
        kinds("edge a -> b"),
        vec![
            TokKind::Ident,
            TokKind::Ident,
            TokKind::Arrow,
            TokKind::Ident
        ]
    );
}

#[test]
fn option_splits_into_key_and_value() {
    assert_eq!(
        kinds("node a fill=blue"),
        vec![TokKind::Ident, TokKind::Ident, TokKind::Key, TokKind::Ident]
    );
    assert_eq!(
        texts("node a fill=blue").last().map(String::as_str),
        Some("blue")
    );
    assert_eq!(
        kinds("node a at=1,-2"),
        vec![TokKind::Ident, TokKind::Ident, TokKind::Key, TokKind::Pair]
    );
    assert_eq!(
        kinds("node a doc=\"a b\""),
        vec![TokKind::Ident, TokKind::Ident, TokKind::Key, TokKind::Str]
    );
}

#[test]
fn string_escapes_are_decoded() {
    let tokens = lex("note \"a\\\"b\\\\c\\nd\\te\"", Span::line_start(1)).expect("lex");
    assert_eq!(
        tokens.last().map(|token| token.text.clone()),
        Some("a\"b\\c\nd\te".to_string())
    );
}

#[test]
fn rejects_unknown_escape() {
    let error = lex("note \"\\x\"", Span::line_start(1)).expect_err("escape inválido");
    assert_eq!(error.code.code, "E007");
}

#[test]
fn rejects_unclosed_string_and_tab() {
    assert_eq!(
        lex("note \"aberta", Span::line_start(1))
            .err()
            .map(|error| error.code.code),
        Some("E007")
    );
    assert_eq!(
        lex("node\ta", Span::line_start(1))
            .err()
            .map(|error| error.code.code),
        Some("E007")
    );
}

#[test]
fn columns_are_one_based_in_characters() {
    let tokens = lex("  node a", Span::line_start(1)).expect("lex");
    assert_eq!(
        tokens
            .first()
            .map(|token| (token.span.line, token.span.col)),
        Some((1, 3))
    );
    let tokens = lex("node a shape=rounded", Span::line_start(7)).expect("lex");
    assert_eq!(
        tokens.get(3).map(|token| (token.span.line, token.span.col)),
        Some((7, 14))
    );
}
