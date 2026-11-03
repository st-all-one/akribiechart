//! Tema: tokens de cor, paletas fechadas e contraste WCAG 2.2.
//!
//! As paletas são **fechadas** (ADR-0005 §5.6). `Token::Paper` é a cor do canvas do tema — num tema
//! escuro, "paper" é escuro —, e é por isso que a paleta de cada tema tem oito entradas (paper, ink,
//! muted, blue, green, amber, red, violet) mais o canvas, que coincide com paper.
//!
//! ## O que é garantido e o que é avisado
//!
//! Medido nas paletas de §5.6:
//!
//! | par | pior caso (paper) | pior caso (slate) | mínimo |
//! |---|---|---|---|
//! | `ink` × `fill` (texto) | 9.99:1 | 7.17:1 | 4.5:1 |
//! | `ink` × canvas | 16.67:1 | 15.29:1 | 4.5:1 |
//! | `muted` × `fill` | 4.01:1 | 4.24:1 | 3:1 |
//! | `muted` × canvas | 6.69:1 | 9.04:1 | 3:1 |
//!
//! Os tokens coloridos (blue, green, amber, red, violet) são **acentos**: a paleta de §5.6 os define
//! como tons claros (tema `paper`) ou escuros (tema `slate`), e nenhum deles atinge 3:1 contra o
//! canvas (máximo medido: 2.13:1). Eles são aceitos como `stroke`, mas `akr check` emite `W007`
//! quando o contraste real fica abaixo do mínimo — o desenho continua legível porque a informação
//! nunca depende só de cor (ADR-0005 §7).

use crate::syntax::{FILL_TOKENS, STROKE_TOKENS};

mod contrast;
#[cfg(test)]
mod tests;

pub use contrast::contrast_ratio;

/// Contraste mínimo para texto (WCAG 2.2 AA, texto normal).
pub const MIN_TEXT_CONTRAST: f32 = 4.5;
/// Contraste mínimo para contorno (WCAG 2.2 SC 1.4.11).
pub const MIN_STROKE_CONTRAST: f32 = 3.0;

/// Token de cor do tema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    /// Sem cor (nenhum preenchimento ou nenhum contorno).
    None,
    /// Cor do canvas do tema.
    Paper,
    /// Cor do texto.
    Ink,
    /// Cinza médio.
    Muted,
    /// Azul.
    Blue,
    /// Verde.
    Green,
    /// Âmbar.
    Amber,
    /// Vermelho.
    Red,
    /// Violeta.
    Violet,
}

impl Token {
    /// Todos os tokens, na ordem dos conjuntos fechados.
    pub const ALL: [Self; 9] = [
        Self::None,
        Self::Paper,
        Self::Ink,
        Self::Muted,
        Self::Blue,
        Self::Green,
        Self::Amber,
        Self::Red,
        Self::Violet,
    ];

    /// Nome do token no `.akrc`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Paper => "paper",
            Self::Ink => "ink",
            Self::Muted => "muted",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Amber => "amber",
            Self::Red => "red",
            Self::Violet => "violet",
        }
    }

    /// Token a partir do nome.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|token| token.name() == name)
    }

    /// `true` quando o token aceita preenchimento (o parser já garante; a fachada confirma).
    pub fn is_fill(self) -> bool {
        FILL_TOKENS.contains(&self.name())
    }

    /// `true` quando o token aceita contorno.
    pub fn is_stroke(self) -> bool {
        STROKE_TOKENS.contains(&self.name())
    }
}

/// Paleta de um tema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Nome do tema (`paper`, `slate`).
    pub name: &'static str,
    /// Cor de fundo do canvas.
    pub canvas: &'static str,
    /// Cor do texto.
    pub ink: &'static str,
    /// Cor do contorno neutro.
    pub muted: &'static str,
    /// Hex de cada token (paper, ink, muted, blue, green, amber, red, violet).
    pub palette: [(&'static str, &'static str); 8],
}

/// Tema claro (padrão).
pub const PAPER: Theme = Theme {
    name: "paper",
    canvas: "#ffffff",
    ink: "#1e1e1e",
    muted: "#5c5c5c",
    palette: [
        ("paper", "#ffffff"),
        ("ink", "#1e1e1e"),
        ("muted", "#5c5c5c"),
        ("blue", "#a5d8ff"),
        ("green", "#b2f2bb"),
        ("amber", "#ffec99"),
        ("red", "#ffc9c9"),
        ("violet", "#d0bfff"),
    ],
};

/// Tema escuro.
pub const SLATE: Theme = Theme {
    name: "slate",
    canvas: "#121212",
    ink: "#e8e8e8",
    muted: "#b4b4b4",
    palette: [
        ("paper", "#121212"),
        ("ink", "#e8e8e8"),
        ("muted", "#b4b4b4"),
        ("blue", "#1e3a5f"),
        ("green", "#1e4620"),
        ("amber", "#5c4813"),
        ("red", "#5f1e1e"),
        ("violet", "#3c2f6b"),
    ],
};

/// Tema pelo nome.
pub fn theme(name: &str) -> Option<&'static Theme> {
    match name {
        "paper" => Some(&PAPER),
        "slate" => Some(&SLATE),
        _ => None,
    }
}

impl Theme {
    /// Hex do token no tema (`None` para [`Token::None`]).
    pub fn hex(&self, token: Token) -> Option<&'static str> {
        let name = token.name();
        self.palette
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, hex)| *hex)
    }

    /// Cor de fundo de um elemento: o `fill`, ou o canvas quando `none`.
    pub fn background(&self, fill: Token) -> &'static str {
        self.hex(fill).unwrap_or(self.canvas)
    }
}

/// Token cujo hex corresponde a `hex` (em qualquer tema), comparação sem caixa.
pub fn token_of_hex(hex: &str) -> Option<Token> {
    let needle = hex.to_ascii_lowercase();
    [PAPER, SLATE]
        .iter()
        .flat_map(|theme| theme.palette.iter())
        .find(|(_, value)| value.eq_ignore_ascii_case(&needle))
        .and_then(|(name, _)| Token::from_name(name))
}
