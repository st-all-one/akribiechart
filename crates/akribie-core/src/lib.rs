//! # akribie-core
//!
//! Núcleo puro do compilador de diagramas: `.akrc` → `Chart` → `Scene` → bytes.
//!
//! Não há I/O aqui. Toda interação com o mundo atravessa as portas de [`ports`] e as funções
//! públicas são puras: mesma entrada, mesma saída. Determinismo é contrato (ADR-0001 §2), não
//! coincidência.
//!
//! ## Estágios
//!
//! ```text
//! lex → parse → resolve → layout → paint → emit
//! ```
//!
//! Cada estágio é uma função sobre dados e pode ser testado isoladamente.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod a11y;
pub mod config;
pub mod diag;
pub mod diff;
pub mod doc;
pub mod emit;
pub mod error;
pub mod fmt;
pub mod geometry;
pub mod import;
pub mod layout;
pub mod lex;
pub mod model;
pub mod num;
pub mod paint;
pub mod parse;
pub mod pipeline;
pub mod ports;
pub mod resolve;
pub mod sketch;
pub mod span;
pub mod syntax;
pub mod text;
pub mod theme;

pub use config::Config;
pub use diag::{Diagnostic, Severity};
pub use diff::{Change, Kind as ChangeKind, diff};
pub use emit::{RasterFormat, encode, excalidraw, scene_json, svg};
pub use error::{Error, ErrorKind, Result};
pub use fmt::format;
pub use geometry::Rect;
pub use layout::{Placed, PlacedNode};
pub use model::{Chart, Chart as ResolvedChart};
pub use paint::Scene;
pub use parse::{Document, parse};
pub use pipeline::{BatchLine, CheckReport, batch_lines, check};

/// Versão do crate, usada em `scene`, `.excalidraw` e no envelope de saída.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
