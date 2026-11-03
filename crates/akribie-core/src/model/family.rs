//! Família tipográfica do documento.

/// Família tipográfica do desenho.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontFamily {
    /// Excalifont, a face manual (padrão).
    #[default]
    Hand,
    /// Liberation Sans, face técnica sem serifa.
    Sans,
}

impl FontFamily {
    /// A partir do nome declarado.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "hand" => Self::Hand,
            "sans" => Self::Sans,
            _ => return None,
        })
    }

    /// Nome no `.akrc`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Hand => "hand",
            Self::Sans => "sans",
        }
    }
}
