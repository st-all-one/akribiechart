//! Opções `chave=valor` de um statement.

use super::{Cursor, malformed};
use crate::diag::{Diagnostic, E002, E003};
use crate::lex::{TokKind, Token};
use crate::syntax::{self, Options, StmtKind};

pub(super) fn parse(
    cursor: &mut Cursor<'_>,
    kind: StmtKind,
    diagnostics: &mut Vec<Diagnostic>,
) -> Options {
    let mut options = Options::new();
    while let Some(token) = cursor.next() {
        if token.kind != TokKind::Key {
            diagnostics.push(malformed(
                token.span,
                format!("esperava `chave=valor`, obteve `{}`", token.text),
                "opções têm a forma `chave=valor`, sem espaços em volta do `=`",
            ));
            continue;
        }
        let Some(spec) = syntax::spec_for(kind, &token.text) else {
            diagnostics.push(unknown_key(kind, token));
            if cursor.peek().is_some_and(|next| next.kind != TokKind::Key) {
                cursor.next();
            }
            continue;
        };
        let Some(value) = cursor.next() else {
            diagnostics.push(malformed(
                token.span,
                format!("a chave `{}` está sem valor", spec.key),
                format!("use `{}={}`", spec.key, spec.default.unwrap_or("<valor>")),
            ));
            continue;
        };
        match syntax::validate(spec, value) {
            Ok(parsed) => {
                options.insert(spec.key, parsed);
            }
            Err(error) => diagnostics.push(
                Diagnostic::new(E003, value.span, error.message).with_optional_hint(error.hint),
            ),
        }
    }
    options
}

fn unknown_key(kind: StmtKind, token: &Token) -> Diagnostic {
    Diagnostic::new(
        E002,
        token.span,
        format!("chave `{}` não existe em `{}`", token.text, kind.keyword()),
    )
    .with_hint(format!(
        "chaves de `{}`: {}",
        kind.keyword(),
        syntax::valid_keys(kind)
    ))
}
