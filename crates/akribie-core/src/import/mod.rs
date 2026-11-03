//! Importação de formatos externos para `.akrc` (ADR-0004).
//!
//! Importação é **best-effort com perda anunciada**: o que não tem representação no `.akrc` vira
//! `W006` e as cores sem token viram `W005`.

pub mod excalidraw;

pub use excalidraw::{Imported, import};
