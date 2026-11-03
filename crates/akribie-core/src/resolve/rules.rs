//! Mensagens de referência e posição (`E005`, `E006`).

use crate::diag::{Diagnostic, E005, E006};
use crate::span::Span;

pub(super) fn missing_group(id: &str, span: Span) -> Diagnostic {
    Diagnostic::new(
        E005,
        span,
        format!("`{id}` em `in=` não é um grupo declarado"),
    )
    .with_hint("`in=` aceita ids de `group`; declare o grupo antes ou depois do nó")
}

pub(super) fn missing_node(id: &str, span: Span, what: &str) -> Diagnostic {
    Diagnostic::new(E005, span, format!("`{id}` não é um nó declarado ({what})"))
        .with_hint("referências exigem um `node` com esse id")
}

pub(super) fn missing_position(id: &str, span: Span) -> Diagnostic {
    Diagnostic::new(E006, span, format!("`{id}` sem `at=` em `layout=free`"))
        .with_hint("em `layout=free` toda posição é declarada: `at=120,80`")
}
