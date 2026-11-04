//! Tabelas de opções por statement — a fonte única da gramática.

use super::{OptSpec, StmtKind, ValKind};

const fn plain(key: &'static str, kind: ValKind) -> OptSpec {
    OptSpec {
        key,
        kind,
        default: None,
        allowed: &[],
        range: None,
    }
}

const fn enumeration(
    key: &'static str,
    default: &'static str,
    allowed: &'static [&'static str],
) -> OptSpec {
    OptSpec {
        key,
        kind: ValKind::Enum,
        default: Some(default),
        allowed,
        range: None,
    }
}

const fn tokens(
    key: &'static str,
    default: &'static str,
    allowed: &'static [&'static str],
) -> OptSpec {
    OptSpec {
        key,
        kind: ValKind::Token,
        default: Some(default),
        allowed,
        range: None,
    }
}

const fn ranged(key: &'static str, default: &'static str, min: f32, max: f32) -> OptSpec {
    OptSpec {
        key,
        kind: ValKind::Number,
        default: Some(default),
        allowed: &[],
        range: Some((min, max)),
    }
}

const fn worded(key: &'static str, default: &'static str) -> OptSpec {
    OptSpec {
        key,
        kind: ValKind::Bareword,
        default: Some(default),
        allowed: &[],
        range: None,
    }
}

/// Conjunto fechado de `layout`.
pub const LAYOUTS: &[&str] = &["flow", "tree", "mindmap", "grid", "stack", "free"];
/// Conjunto fechado de `direction`.
pub const DIRECTIONS: &[&str] = &["lr", "tb", "bt", "rl"];
/// Conjunto fechado de `theme`.
pub const THEMES: &[&str] = &["paper", "slate"];
/// Conjunto fechado de `shape`.
pub const SHAPES: &[&str] = &[
    "rect", "rounded", "circle", "diamond", "cloud", "cylinder", "hexagon", "note", "text",
];
/// Tokens aceitos em `fill` (`ink` e `muted` ficam de fora por contraste — ADR-0005).
pub const FILL_TOKENS: &[&str] = &["none", "paper", "blue", "green", "amber", "red", "violet"];
/// Tokens aceitos em `stroke`.
pub const STROKE_TOKENS: &[&str] = &[
    "none", "paper", "ink", "muted", "blue", "green", "amber", "red", "violet",
];
/// Conjunto fechado de `fillstyle`.
pub const FILL_STYLES: &[&str] = &["hachure", "solid", "cross-hatch", "zigzag"];
/// Conjunto fechado de `style`.
pub const STROKE_STYLES: &[&str] = &["solid", "dashed", "dotted"];
/// Conjunto fechado de `curve`.
pub const CURVES: &[&str] = &["line", "arc", "elbow"];
/// Conjunto fechado de `head` e `tail`.
pub const HEADS: &[&str] = &["arrow", "none"];
/// Conjunto fechado de `sketch`.
pub const SKETCHES: &[&str] = &["on", "off"];
/// Conjunto fechado de `font`.
pub const FONTS: &[&str] = &["hand", "sans"];

const CHART: &[OptSpec] = &[
    enumeration("layout", "flow", LAYOUTS),
    enumeration("direction", "lr", DIRECTIONS),
    enumeration("theme", "paper", THEMES),
    ranged("fontsize", "20", 14.0, 96.0),
    ranged("pad", "32", 0.0, 256.0),
    worded("lang", "pt-BR"),
    enumeration("sketch", "off", SKETCHES),
    enumeration("font", "sans", FONTS),
    plain("doc", ValKind::Str),
];

const GROUP: &[OptSpec] = &[
    tokens("fill", "none", FILL_TOKENS),
    tokens("stroke", "ink", STROKE_TOKENS),
    enumeration("style", "solid", STROKE_STYLES),
    ranged("width", "2", 2.0, 6.0),
    ranged("pad", "24", 0.0, 256.0),
    plain("in", ValKind::IdList),
];

const NODE: &[OptSpec] = &[
    enumeration("shape", "rounded", SHAPES),
    tokens("fill", "none", FILL_TOKENS),
    enumeration("fillstyle", "hachure", FILL_STYLES),
    tokens("stroke", "ink", STROKE_TOKENS),
    enumeration("style", "solid", STROKE_STYLES),
    ranged("width", "2", 2.0, 6.0),
    plain("in", ValKind::IdList),
    plain("at", ValKind::Pair),
    OptSpec {
        range: Some((1.0, f32::MAX)),
        ..plain("size", ValKind::Pair)
    },
];

const EDGE: &[OptSpec] = &[
    enumeration("curve", "line", CURVES),
    enumeration("head", "arrow", HEADS),
    enumeration("tail", "none", HEADS),
    enumeration("style", "solid", STROKE_STYLES),
    tokens("stroke", "ink", STROKE_TOKENS),
    ranged("width", "2", 2.0, 6.0),
];

const NOTE: &[OptSpec] = &[
    tokens("fill", "amber", FILL_TOKENS),
    plain("at", ValKind::Pair),
    OptSpec {
        range: Some((1.0, f32::MAX)),
        ..plain("size", ValKind::Pair)
    },
    plain("near", ValKind::Ident),
];

/// Tabela de opções do statement — a fonte única da gramática.
pub const fn option_specs(kind: StmtKind) -> &'static [OptSpec] {
    match kind {
        StmtKind::Chart => CHART,
        StmtKind::Group => GROUP,
        StmtKind::Node => NODE,
        StmtKind::Edge => EDGE,
        StmtKind::Note => NOTE,
    }
}

/// Spec da chave, se ela pertencer ao statement.
pub fn spec_for(kind: StmtKind, key: &str) -> Option<&'static OptSpec> {
    option_specs(kind).iter().find(|spec| spec.key == key)
}

/// Lista de chaves válidas do statement, para mensagens de erro.
pub fn valid_keys(kind: StmtKind) -> String {
    option_specs(kind)
        .iter()
        .map(|spec| spec.key)
        .collect::<Vec<_>>()
        .join(", ")
}
