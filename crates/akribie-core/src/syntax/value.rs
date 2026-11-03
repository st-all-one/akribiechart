//! Validação de valores contra a spec da chave.

use super::{OptSpec, OptVal, ValKind, ValueError};
use crate::lex::{TokKind, Token};
use crate::num;
use crate::span::Span;

fn invalid(message: impl Into<String>, hint: impl Into<String>) -> ValueError {
    ValueError {
        message: message.into(),
        hint: Some(hint.into()),
    }
}

/// `true` se o texto é um `IDENT` (`[A-Za-z][A-Za-z0-9_-]{0,63}`).
pub fn is_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && text.chars().count() <= 64
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// `true` se o texto é um BCP-47 simples (`en`, `pt-BR`, `es-419`).
pub fn is_language_tag(text: &str) -> bool {
    let mut parts = text.split('-');
    let Some(primary) = parts.next() else {
        return false;
    };
    let primary_len = primary.chars().count();
    if !(2..=3).contains(&primary_len) || !primary.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    parts.all(|part| {
        let length = part.chars().count();
        (2..=8).contains(&length) && part.chars().all(|c| c.is_ascii_alphanumeric())
    })
}

fn is_word(kind: TokKind) -> bool {
    matches!(kind, TokKind::Ident | TokKind::Version | TokKind::Bare)
}

fn parse_number(text: &str, span: Span) -> Result<f32, ValueError> {
    match text.parse::<f32>() {
        Ok(value) if value.is_finite() => Ok(value),
        _ => Err(invalid(
            format!("`{text}` não é um número"),
            format!(
                "use um número decimal na linha {}, coluna {}, como 20 ou 2.5",
                span.line, span.col
            ),
        )),
    }
}

fn check_range(spec: &OptSpec, value: f32) -> Result<f32, ValueError> {
    match spec.range {
        Some((min, max)) if value < min || value > max => Err(invalid(
            format!(
                "`{}` está fora da faixa de `{}`",
                num::format(value),
                spec.key
            ),
            format!(
                "`{}` aceita de {} a {}",
                spec.key,
                num::format(min),
                num::format(max)
            ),
        )),
        _ => Ok(value),
    }
}

/// Valida o token de valor contra a spec e devolve o valor tipado.
pub fn validate(spec: &OptSpec, token: &Token) -> Result<OptVal, ValueError> {
    match spec.kind {
        ValKind::Enum | ValKind::Token => {
            if is_word(token.kind) && spec.allowed.contains(&token.text.as_str()) {
                Ok(OptVal::Bare(token.text.clone()))
            } else {
                Err(invalid(
                    format!("`{}={}` não é um valor válido", spec.key, token.text),
                    format!("valores de `{}`: {}", spec.key, spec.allowed.join(", ")),
                ))
            }
        }
        ValKind::Number => {
            if !is_word(token.kind) {
                return Err(invalid(
                    format!("`{}` espera um número", spec.key),
                    "use um número decimal, como 20 ou 2.5",
                ));
            }
            let value = parse_number(&token.text, token.span)?;
            Ok(OptVal::Num(check_range(spec, value)?))
        }
        ValKind::Str => {
            if token.kind == TokKind::Str {
                Ok(OptVal::Str(token.text.clone()))
            } else {
                Err(invalid(
                    format!("`{}` espera uma string entre aspas", spec.key),
                    format!("escreva {}=\"{}\"", spec.key, token.text),
                ))
            }
        }
        ValKind::Bareword => {
            if is_word(token.kind) && is_language_tag(&token.text) {
                Ok(OptVal::Bare(token.text.clone()))
            } else {
                Err(invalid(
                    format!("`{}={}` não é uma etiqueta de idioma", spec.key, token.text),
                    "use BCP-47 simples, como en, pt-BR ou es",
                ))
            }
        }
        ValKind::Ident => {
            if is_word(token.kind) && is_ident(&token.text) {
                Ok(OptVal::Bare(token.text.clone()))
            } else {
                Err(invalid(
                    format!("`{}={}` não é um identificador", spec.key, token.text),
                    "identificadores começam com letra e usam letras, dígitos, `_` ou `-`",
                ))
            }
        }
        ValKind::IdList => {
            if !matches!(
                token.kind,
                TokKind::Ident | TokKind::Version | TokKind::Pair
            ) {
                return Err(invalid(
                    format!("`{}` espera ids separados por vírgula", spec.key),
                    "use `in=g1` ou `in=g1,g2`, sem espaços",
                ));
            }
            let ids: Vec<String> = token.text.split(',').map(str::to_string).collect();
            if ids.iter().all(|id| is_ident(id)) {
                Ok(OptVal::Ids(ids))
            } else {
                Err(invalid(
                    format!("`{}` contém um id inválido", spec.key),
                    "cada id começa com letra e usa letras, dígitos, `_` ou `-`",
                ))
            }
        }
        ValKind::Pair => {
            if token.kind != TokKind::Pair {
                return Err(invalid(
                    format!("`{}` espera o par `x,y`", spec.key),
                    format!("escreva {}={},{}", spec.key, 0, 0),
                ));
            }
            let Some((raw_x, raw_y)) = token.text.split_once(',') else {
                return Err(invalid(
                    format!("`{}` espera o par `x,y`", spec.key),
                    "separe os dois números por uma vírgula, sem espaços",
                ));
            };
            let x = check_range(spec, parse_number(raw_x, token.span)?)?;
            let y = check_range(spec, parse_number(raw_y, token.span)?)?;
            Ok(OptVal::Pair(x, y))
        }
    }
}
