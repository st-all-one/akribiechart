//! Tokenizador de uma linha do `.akrc`.
//!
//! Uma linha é uma sequência de tokens separados por **um ou mais espaços**. Tab é erro (`E007`),
//! strings podem conter espaços, e `chave=valor` é um único grupo indivisível: o `=` não é
//! separador, é parte do token — por isso `layout = flow` não é aceito e `layout=flow` é.

use crate::diag::{Diagnostic, E007};
use crate::span::Span;

/// Espécie de token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokKind {
    /// Identificador ou palavra-chave (`node`, `api`, `pt-BR`).
    Ident,
    /// Versão do documento (`v1`).
    Version,
    /// String decodificada (sem aspas, com escapes resolvidos).
    Str,
    /// Nome de chave de opção (`layout` de `layout=flow`).
    Key,
    /// Palavra numérica ou literal (`20`, `1.5`, `-3`).
    Bare,
    /// A seta `->`.
    Arrow,
    /// Grupo com vírgula (`1,2` ou `g1,g2`), validado pelo parser.
    Pair,
}

/// Token com posição.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Espécie.
    pub kind: TokKind,
    /// Texto (strings já decodificadas).
    pub text: String,
    /// Posição do primeiro caractere.
    pub span: Span,
}

/// Tokeniza uma linha. Devolve `E007` na primeira irregularidade da linha.
pub fn lex(line: &str, start: Span) -> Result<Vec<Token>, Diagnostic> {
    let mut scanner = Scanner::new(line, start);
    let mut tokens = Vec::new();
    loop {
        scanner.skip_spaces();
        let Some(first) = scanner.peek() else { break };
        if first == '\t' {
            return Err(scanner.malformed(
                "tab não é separador válido",
                "substitua o tab por um ou mais espaços",
            ));
        }
        let (raw, span) = scanner.read_word()?;
        classify(&raw, span, &mut tokens)?;
    }
    Ok(tokens)
}

fn classify(raw: &str, span: Span, out: &mut Vec<Token>) -> Result<(), Diagnostic> {
    // Um token que abre com aspas é uma string inteira: `=` dentro dela é conteúdo
    // (`node a "user=admin"`), nunca separador de chave.
    if raw.starts_with('"') {
        out.push(single(raw, span)?);
        return Ok(());
    }
    let Some((key, value)) = raw.split_once('=') else {
        out.push(single(raw, span)?);
        return Ok(());
    };
    if !is_key(key) {
        return Err(
            Diagnostic::new(E007, span, format!("`{key}` não é um nome de chave válido"))
                .with_hint("chaves são `[a-z][a-z0-9_]{0,31}`, como layout, fill ou fontsize"),
        );
    }
    if value.is_empty() {
        return Err(
            Diagnostic::new(E007, span, format!("a opção `{key}=` está sem valor")).with_hint(
                format!("escreva `{key}=<valor>`; o `=` não pode ser seguido de espaço"),
            ),
        );
    }
    let key_len = u32::try_from(key.chars().count()).unwrap_or(u32::MAX);
    let value_span = Span::new(
        span.line,
        span.col.saturating_add(key_len).saturating_add(1),
    );
    out.push(Token {
        kind: TokKind::Key,
        text: key.to_string(),
        span,
    });
    out.push(single(value, value_span)?);
    Ok(())
}

fn single(raw: &str, span: Span) -> Result<Token, Diagnostic> {
    let kind = if raw == "->" {
        TokKind::Arrow
    } else if raw.starts_with('"') {
        TokKind::Str
    } else if is_version(raw) {
        TokKind::Version
    } else if is_ident(raw) {
        TokKind::Ident
    } else if raw.contains(',') {
        TokKind::Pair
    } else if is_bare(raw) {
        TokKind::Bare
    } else {
        return Err(
            Diagnostic::new(E007, span, format!("token inválido `{raw}`")).with_hint(
                "valores são palavras, números, strings entre aspas, `chave=valor` ou `a,b`",
            ),
        );
    };
    let text = if kind == TokKind::Str {
        decode_string(raw, span)?
    } else {
        raw.to_string()
    };
    Ok(Token { kind, text, span })
}

fn decode_string(raw: &str, span: Span) -> Result<String, Diagnostic> {
    let Some(inner) = raw
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    else {
        return Err(
            Diagnostic::new(E007, span, "string com aspas desbalanceadas")
                .with_hint("cada `\"` de abertura precisa de um `\"` de fechamento"),
        );
    };
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let Some(escaped) = chars.next() else {
            return Err(
                Diagnostic::new(E007, span, "barra invertida no fim da string")
                    .with_hint("use `\\\\` para uma barra literal"),
            );
        };
        match escaped {
            '"' => out.push('"'),
            '\\' => out.push('\\'),
            'n' => out.push('\n'),
            't' => out.push('\t'),
            other => {
                return Err(Diagnostic::new(
                    E007,
                    span,
                    format!("escape `\\{other}` não é aceito"),
                )
                .with_hint("escapes aceitos: \\\" \\\\ \\n \\t"));
            }
        }
    }
    Ok(out)
}

/// `[A-Za-z][A-Za-z0-9_-]{0,63}`.
fn is_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && text.chars().count() <= 64
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// `v` seguido de dígitos.
fn is_version(text: &str) -> bool {
    match text.strip_prefix('v') {
        Some(digits) => !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()),
        None => false,
    }
}

/// `[a-z][a-z0-9_]{0,31}`.
fn is_key(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_lowercase()
        && text.chars().count() <= 32
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// `[A-Za-z0-9_.-]+` (aceita números negativos e decimais).
fn is_bare(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')
}

fn push_escape(scanner: &mut Scanner, raw: &mut String) {
    scanner.bump();
    raw.push('\\');
    if let Some(escaped) = scanner.bump() {
        raw.push(escaped);
    }
}

struct Scanner {
    chars: Vec<char>,
    at: usize,
    line: u32,
    base_col: u32,
}

impl Scanner {
    fn new(line: &str, start: Span) -> Self {
        Self {
            chars: line.chars().collect(),
            at: 0,
            line: start.line,
            base_col: start.col,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let value = self.peek();
        if value.is_some() {
            self.at = self.at.saturating_add(1);
        }
        value
    }

    fn span(&self) -> Span {
        let offset = u32::try_from(self.at).unwrap_or(u32::MAX);
        Span::new(self.line, self.base_col.saturating_add(offset))
    }

    fn skip_spaces(&mut self) {
        while self.peek() == Some(' ') {
            self.bump();
        }
    }

    fn malformed(&self, message: &str, hint: &str) -> Diagnostic {
        Diagnostic::new(E007, self.span(), message).with_hint(hint)
    }

    /// Lê até o próximo espaço fora de string; devolve o texto bruto (com aspas).
    fn read_word(&mut self) -> Result<(String, Span), Diagnostic> {
        let span = self.span();
        let mut raw = String::new();
        let mut in_string = false;
        while let Some(c) = self.peek() {
            if c == ' ' && !in_string {
                break;
            }
            if c == '\t' {
                return Err(self.malformed(
                    "tab não é separador válido",
                    "substitua o tab por um ou mais espaços",
                ));
            }
            if c == '\\' && in_string {
                push_escape(self, &mut raw);
                continue;
            }
            if c == '"' {
                in_string = !in_string;
            }
            raw.push(c);
            self.bump();
        }
        if in_string {
            return Err(
                Diagnostic::new(E007, span, "string sem aspas de fechamento")
                    .with_hint("feche a string com `\"`; escapes aceitos: \\\" \\\\ \\n \\t"),
            );
        }
        Ok((raw, span))
    }
}

#[cfg(test)]
mod tests;
