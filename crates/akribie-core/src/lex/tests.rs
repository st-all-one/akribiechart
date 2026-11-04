//! Testes do léxico: separação de `chave=valor` × conteúdo de string.

use super::{TokKind, lex};
use crate::span::Span;

fn kinds(line: &str) -> Vec<(TokKind, String)> {
    let tokens = lex(line, Span::line_start(1)).expect("linha de teste deveria tokenizar");
    tokens
        .into_iter()
        .map(|token| (token.kind, token.text))
        .collect()
}

#[test]
fn quoted_label_keeps_equals_as_content() {
    let tokens = kinds("node a \"user=admin\"");
    assert_eq!(
        tokens,
        [
            (TokKind::Ident, "node".to_string()),
            (TokKind::Ident, "a".to_string()),
            (TokKind::Str, "user=admin".to_string()),
        ]
    );
}

#[test]
fn quoted_edge_label_keeps_equals_as_content() {
    let tokens = kinds("edge a -> b \"x=1\"");
    let label = tokens.last().expect("rótulo");
    assert_eq!(label.0, TokKind::Str);
    assert_eq!(label.1, "x=1");
}

#[test]
fn unquoted_option_still_splits_on_equals() {
    let tokens = kinds("chart v1 \"T\" layout=flow");
    let key = &tokens[tokens.len() - 2];
    let value = &tokens[tokens.len() - 1];
    assert_eq!(key, &(TokKind::Key, "layout".to_string()));
    assert_eq!(value, &(TokKind::Ident, "flow".to_string()));
}

#[test]
fn quoted_value_after_key_is_one_string() {
    let tokens = kinds("chart v1 \"T\" doc=\"README.md#a=b\"");
    let key = &tokens[tokens.len() - 2];
    let value = &tokens[tokens.len() - 1];
    assert_eq!(key, &(TokKind::Key, "doc".to_string()));
    assert_eq!(value, &(TokKind::Str, "README.md#a=b".to_string()));
}
