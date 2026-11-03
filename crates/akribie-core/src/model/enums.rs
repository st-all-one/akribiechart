//! Enums fechados do modelo (um por conjunto do `.akrc`).

/// Perfil de layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Fluxo em camadas (dagre).
    Flow,
    /// Árvore (Reingold–Tilford).
    Tree,
    /// Mapa mental radial.
    Mindmap,
    /// Grade quadrada.
    Grid,
    /// Pilha vertical.
    Stack,
    /// Posições declaradas.
    Free,
}

impl Layout {
    /// A partir do nome declarado.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "flow" => Self::Flow,
            "tree" => Self::Tree,
            "mindmap" => Self::Mindmap,
            "grid" => Self::Grid,
            "stack" => Self::Stack,
            "free" => Self::Free,
            _ => return None,
        })
    }

    /// Nome no `.akrc`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Flow => "flow",
            Self::Tree => "tree",
            Self::Mindmap => "mindmap",
            Self::Grid => "grid",
            Self::Stack => "stack",
            Self::Free => "free",
        }
    }

    /// `true` quando todo `node`/`note` precisa de `at`.
    pub const fn requires_position(self) -> bool {
        matches!(self, Self::Free)
    }

    /// `true` quando um nó sem aresta é suspeito (`W002`).
    pub const fn expects_graph(self) -> bool {
        matches!(self, Self::Flow | Self::Tree | Self::Mindmap)
    }
}

/// Direção do fluxo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Esquerda → direita.
    Lr,
    /// Cima → baixo.
    Tb,
    /// Baixo → cima.
    Bt,
    /// Direita → esquerda.
    Rl,
}

impl Direction {
    /// A partir do nome declarado.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "lr" => Self::Lr,
            "tb" => Self::Tb,
            "bt" => Self::Bt,
            "rl" => Self::Rl,
            _ => return None,
        })
    }

    /// Nome no `.akrc`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Lr => "lr",
            Self::Tb => "tb",
            Self::Bt => "bt",
            Self::Rl => "rl",
        }
    }
}

/// Forma do elemento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// Retângulo.
    Rect,
    /// Retângulo arredondado (padrão).
    Rounded,
    /// Círculo/elipse.
    Circle,
    /// Losango (decisão).
    Diamond,
    /// Nuvem (externo/lento).
    Cloud,
    /// Cilindro (armazenamento).
    Cylinder,
    /// Hexágono.
    Hexagon,
    /// Nota com canto dobrado.
    Note,
    /// Só texto.
    Text,
}

impl Shape {
    /// A partir do nome declarado.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "rect" => Self::Rect,
            "rounded" => Self::Rounded,
            "circle" => Self::Circle,
            "diamond" => Self::Diamond,
            "cloud" => Self::Cloud,
            "cylinder" => Self::Cylinder,
            "hexagon" => Self::Hexagon,
            "note" => Self::Note,
            "text" => Self::Text,
            _ => return None,
        })
    }

    /// Nome no `.akrc`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Rect => "rect",
            Self::Rounded => "rounded",
            Self::Circle => "circle",
            Self::Diamond => "diamond",
            Self::Cloud => "cloud",
            Self::Cylinder => "cylinder",
            Self::Hexagon => "hexagon",
            Self::Note => "note",
            Self::Text => "text",
        }
    }

    /// `true` quando a forma não desenha contorno (só texto).
    pub const fn is_outline_only(self) -> bool {
        matches!(self, Self::Text)
    }
}

/// Estilo de preenchimento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillStyle {
    /// Hachurado a 45° (padrão).
    Hachure,
    /// Cor sólida.
    Solid,
    /// Hachura cruzada (±45°).
    CrossHatch,
    /// Zigue-zague.
    Zigzag,
}

impl FillStyle {
    /// A partir do nome declarado.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "hachure" => Self::Hachure,
            "solid" => Self::Solid,
            "cross-hatch" => Self::CrossHatch,
            "zigzag" => Self::Zigzag,
            _ => return None,
        })
    }

    /// Nome no `.akrc`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Hachure => "hachure",
            Self::Solid => "solid",
            Self::CrossHatch => "cross-hatch",
            Self::Zigzag => "zigzag",
        }
    }
}

/// Estilo de traço.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeStyle {
    /// Contínuo (padrão).
    Solid,
    /// Tracejado.
    Dashed,
    /// Pontilhado.
    Dotted,
}

impl StrokeStyle {
    /// A partir do nome declarado.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "solid" => Self::Solid,
            "dashed" => Self::Dashed,
            "dotted" => Self::Dotted,
            _ => return None,
        })
    }

    /// Nome no `.akrc`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Dashed => "dashed",
            Self::Dotted => "dotted",
        }
    }

    /// `stroke-dasharray` do SVG, quando houver.
    pub const fn dasharray(self) -> Option<&'static str> {
        match self {
            Self::Solid => None,
            Self::Dashed => Some("8 8"),
            Self::Dotted => Some("1.5 6"),
        }
    }
}

/// Curva da aresta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Curve {
    /// Segmento reto (padrão).
    Line,
    /// Arco quadrático.
    Arc,
    /// Cotovelo H-V-H / V-H-V.
    Elbow,
}

impl Curve {
    /// A partir do nome declarado.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "line" => Self::Line,
            "arc" => Self::Arc,
            "elbow" => Self::Elbow,
            _ => return None,
        })
    }

    /// Nome no `.akrc`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Arc => "arc",
            Self::Elbow => "elbow",
        }
    }
}

/// Ponta da aresta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Head {
    /// Seta (padrão em `head`).
    Arrow,
    /// Sem ponta.
    None,
}

impl Head {
    /// A partir do nome declarado.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "arrow" => Self::Arrow,
            "none" => Self::None,
            _ => return None,
        })
    }

    /// Nome no `.akrc`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Arrow => "arrow",
            Self::None => "none",
        }
    }
}
