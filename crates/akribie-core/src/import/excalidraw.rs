//! `.excalidraw` → `.akrc`.
//!
//! Mapeamento do ADR-0004: retângulo → `shape=rounded`, elipse → `circle`, losango → `diamond`,
//! texto com `containerId` → rótulo do container, texto solto → `note`, seta com os dois bindings →
//! `edge`. O resto é descartado com `W006`. A rota nunca é importada: o layout recalcula.

use serde_json::Value;

mod collect;
mod render;

use self::collect::collect;
use self::render::render;

use crate::diag::Diagnostic;
use crate::error::{Error, Result};
use crate::fmt;
use crate::parse::parse;

/// Resultado da importação.
#[derive(Debug, Clone, PartialEq)]
pub struct Imported {
    /// Fonte `.akrc` na forma canônica.
    pub source: String,
    /// Perdas anunciadas (`W005`/`W006`).
    pub diagnostics: Vec<Diagnostic>,
}

/// Converte um `.excalidraw` em `.akrc`.
///
/// # Errors
/// `InvalidInput` quando o JSON não for um objeto, não tiver `type: "excalidraw"` ou não tiver
/// `elements`.
pub fn import(input: &str, name: &str) -> Result<Imported> {
    let value: Value = serde_json::from_str(input)
        .map_err(|error| Error::invalid_input(format!("JSON inválido: {error}")))?;
    if value.get("type").and_then(Value::as_str) != Some("excalidraw") {
        return Err(Error::invalid_input(
            "não é um `.excalidraw`: falta `\"type\": \"excalidraw\"`",
        ));
    }
    let elements: Vec<&Value> = value
        .get("elements")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::invalid_input("`.excalidraw` sem `elements`"))?
        .iter()
        .filter(|element| {
            !element
                .get("isDeleted")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .collect();

    let (diagnostics, tables) = collect(&elements);
    let (document, _) = parse(&render(name, &tables));
    Ok(Imported {
        source: fmt::format(&document),
        diagnostics,
    })
}

#[cfg(test)]
mod tests;
