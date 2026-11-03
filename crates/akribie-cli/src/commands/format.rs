//! Formato de saída do `render` (texto para `stdout`, raster exige `--out`).

use akribie_core::emit::RasterFormat;
use akribie_core::error::{Error, Result};

/// Formato de saída.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Format {
    /// SVG (texto, pode ir para `stdout`).
    Svg,
    /// `scene` JSON (texto, pode ir para `stdout`).
    Scene,
    /// `.excalidraw` (texto, pode ir para `stdout`).
    Excalidraw,
    /// Raster (binário, exige `--out`).
    Raster(RasterFormat),
}

impl Format {
    pub(crate) fn parse(name: &str) -> Result<Self> {
        match name {
            "svg" => return Ok(Self::Svg),
            "scene" => return Ok(Self::Scene),
            "excalidraw" => return Ok(Self::Excalidraw),
            _ => {}
        }
        RasterFormat::from_name(name)
            .map(Self::Raster)
            .ok_or_else(|| {
                Error::invalid_input(format!(
                    "formato `{name}` desconhecido; use svg, png, jpg ou avif"
                ))
            })
    }

    /// Extensão de arquivo do formato.
    pub(crate) const fn extension(self) -> &'static str {
        match self {
            Self::Svg => "svg",
            Self::Scene => "json",
            Self::Excalidraw => "excalidraw",
            Self::Raster(raster) => raster.extension(),
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Svg => "svg",
            Self::Scene => "scene",
            Self::Excalidraw => "excalidraw",
            Self::Raster(raster) => raster.extension(),
        }
    }
}
