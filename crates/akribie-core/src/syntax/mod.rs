//! Gramática do `.akrc`: tipos, tabelas de opções e validação de valores.
//!
//! As tabelas de [`specs`] são a **única** fonte de chaves, tipos, defaults, faixas e conjuntos
//! fechados. O parser valida por elas; `akr schema` e `akr prime` as serializam. Chave nova entra
//! aqui e aparece nos três consumidores no mesmo passo (ADR-0002).

mod specs;
mod value;

pub use specs::{
    CURVES, DIRECTIONS, FILL_STYLES, FILL_TOKENS, FONTS, HEADS, LAYOUTS, SHAPES, SKETCHES,
    STROKE_STYLES, STROKE_TOKENS, THEMES, option_specs, spec_for, valid_keys,
};
pub use value::{is_ident, is_language_tag, validate};

use indexmap::IndexMap;

use crate::num;
use crate::span::Span;

/// Conjunto de opções de um statement, com a ordem de declaração.
pub type Options = IndexMap<&'static str, OptVal>;

/// Classe de statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StmtKind {
    /// `chart v1 "Título" …`
    Chart,
    /// `group ID "Rótulo" …`
    Group,
    /// `node ID "Rótulo" …`
    Node,
    /// `edge FROM -> TO "Rótulo" …`
    Edge,
    /// `note "Texto" …`
    Note,
}

impl StmtKind {
    /// Palavra-chave do statement.
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Chart => "chart",
            Self::Group => "group",
            Self::Node => "node",
            Self::Edge => "edge",
            Self::Note => "note",
        }
    }
}

/// Tipo de valor aceito por uma chave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValKind {
    /// Um valor do conjunto `allowed`.
    Enum,
    /// Número decimal (inteiro ou com ponto).
    Number,
    /// Token de tema, do conjunto `allowed`.
    Token,
    /// String entre aspas.
    Str,
    /// Palavra (BCP-47 simples, para `lang`).
    Bareword,
    /// Lista de ids separada por vírgula, sem espaços.
    IdList,
    /// Par ordenado `x,y` de números.
    Pair,
    /// Um identificador (referência).
    Ident,
}

impl ValKind {
    /// Nome do tipo em `akr schema`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enum => "enum",
            Self::Number => "number",
            Self::Token => "token",
            Self::Str => "string",
            Self::Bareword => "bareword",
            Self::IdList => "idlist",
            Self::Pair => "pair",
            Self::Ident => "ident",
        }
    }
}

/// Especificação de uma chave de statement.
#[derive(Debug, Clone, Copy)]
pub struct OptSpec {
    /// Nome da chave.
    pub key: &'static str,
    /// Tipo do valor.
    pub kind: ValKind,
    /// Default declarado (texto canônico), quando houver.
    pub default: Option<&'static str>,
    /// Conjunto fechado de valores (vazio para tipos abertos).
    pub allowed: &'static [&'static str],
    /// Faixa numérica permitida, quando houver.
    pub range: Option<(f32, f32)>,
}

/// Valor de uma opção, já validado e tipado.
#[derive(Debug, Clone, PartialEq)]
pub enum OptVal {
    /// String entre aspas.
    Str(String),
    /// Palavra (enum, token ou bareword).
    Bare(String),
    /// Número.
    Num(f32),
    /// Par `x,y`.
    Pair(f32, f32),
    /// Lista de ids.
    Ids(Vec<String>),
}

impl OptVal {
    /// Texto do valor como `&str` (apenas para variantes textuais).
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(text) | Self::Bare(text) => Some(text),
            Self::Num(_) | Self::Pair(_, _) | Self::Ids(_) => None,
        }
    }

    /// Número, se a variante for numérica.
    pub const fn as_num(&self) -> Option<f32> {
        match self {
            Self::Num(value) => Some(*value),
            Self::Str(_) | Self::Bare(_) | Self::Pair(_, _) | Self::Ids(_) => None,
        }
    }

    /// Par de números, se a variante for um par.
    pub const fn as_pair(&self) -> Option<(f32, f32)> {
        match self {
            Self::Pair(x, y) => Some((*x, *y)),
            Self::Str(_) | Self::Bare(_) | Self::Num(_) | Self::Ids(_) => None,
        }
    }

    /// Lista de ids, se a variante for uma lista.
    pub fn as_ids(&self) -> Option<&[String]> {
        match self {
            Self::Ids(ids) => Some(ids),
            Self::Str(_) | Self::Bare(_) | Self::Num(_) | Self::Pair(_, _) => None,
        }
    }

    /// Texto canônico (usado por `akr fmt`).
    pub fn render(&self) -> String {
        match self {
            Self::Str(text) => format!("\"{}\"", escape_string(text)),
            Self::Bare(text) => text.clone(),
            Self::Num(value) => num::format(*value),
            Self::Pair(x, y) => format!("{},{}", num::format(*x), num::format(*y)),
            Self::Ids(ids) => ids.join(","),
        }
    }
}

/// Falha de validação de um valor: mensagem + correção sugerida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueError {
    /// O que se esperava.
    pub message: String,
    /// Como corrigir.
    pub hint: Option<String>,
}

/// Escapa uma string para a forma canônica (`fmt`) — inverso de `lex`.
pub fn escape_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

/// Leitura de uma opção textual por chave, sem `unwrap` na ponta.
pub fn text<'a>(options: &'a Options, key: &str) -> Option<&'a str> {
    options.get(key).and_then(OptVal::as_str)
}

/// Leitura de uma opção numérica por chave.
pub fn number(options: &Options, key: &str) -> Option<f32> {
    options.get(key).and_then(OptVal::as_num)
}

/// Leitura de uma opção de par por chave.
pub fn pair(options: &Options, key: &str) -> Option<(f32, f32)> {
    options.get(key).and_then(OptVal::as_pair)
}

/// Leitura de uma opção de lista por chave.
pub fn ids<'a>(options: &'a Options, key: &str) -> Option<&'a [String]> {
    options.get(key).and_then(OptVal::as_ids)
}

/// Statement `chart`: título opcional e parâmetros do documento.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartStmt {
    /// Título (vira `<title>` no SVG); ausente ⇒ SVG sem `aria-label`.
    pub title: Option<String>,
    /// Opções declaradas.
    pub options: Options,
    /// Comentários imediatamente anteriores.
    pub comments: Vec<String>,
    /// Posição da palavra-chave.
    pub span: Span,
}

/// Statement `group`: agrupamento visual de nós.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupStmt {
    /// Id único entre `group`, `node` e `note`.
    pub id: String,
    /// Rótulo; ausente ⇒ o id é o rótulo.
    pub label: Option<String>,
    /// Opções declaradas.
    pub options: Options,
    /// Comentários imediatamente anteriores.
    pub comments: Vec<String>,
    /// Posição da palavra-chave.
    pub span: Span,
}

/// Statement `node`: um elemento desenhado com forma e rótulo.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeStmt {
    /// Id único entre `group`, `node` e `note`.
    pub id: String,
    /// Rótulo; ausente ⇒ o id é o rótulo.
    pub label: Option<String>,
    /// Opções declaradas.
    pub options: Options,
    /// Comentários imediatamente anteriores.
    pub comments: Vec<String>,
    /// Posição da palavra-chave.
    pub span: Span,
}

/// Statement `edge`: ligação dirigida entre dois ids.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeStmt {
    /// Id de origem.
    pub from: String,
    /// Id de destino.
    pub to: String,
    /// Rótulo; ausente ⇒ aresta sem rótulo.
    pub label: Option<String>,
    /// Opções declaradas.
    pub options: Options,
    /// Comentários imediatamente anteriores.
    pub comments: Vec<String>,
    /// Posição da palavra-chave.
    pub span: Span,
}

/// Statement `note`: anotação em texto livre.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteStmt {
    /// Texto da nota.
    pub text: String,
    /// Opções declaradas.
    pub options: Options,
    /// Comentários imediatamente anteriores.
    pub comments: Vec<String>,
    /// Posição da palavra-chave.
    pub span: Span,
}
