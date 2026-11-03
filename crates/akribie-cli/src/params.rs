//! `--params`: um objeto JSON por verbo, com validação explícita das chaves.
//!
//! A lista de chaves de cada verbo vive aqui e é a mesma documentada em `docs/usage/`. Chave fora da
//! lista produz erro que **lista as válidas** (um agente corrige sem adivinhar).

use akribie_core::Error;
use akribie_core::error::Result;
use serde::Deserialize;
use serde_json::Value;

/// Chaves válidas de `--params` em `check`.
pub(crate) const CHECK_KEYS: &[&str] = &["path", "source", "strict", "docs"];

/// Chaves válidas de `--params` em `fmt`.
pub(crate) const FMT_KEYS: &[&str] = &["path", "source", "write", "check"];

/// Chaves válidas de `--params` em `render`.
pub(crate) const RENDER_KEYS: &[&str] =
    &["path", "source", "format", "out", "scale", "at", "dry_run"];

/// Chaves aceitas por `akr diff --params`.
pub(crate) const DIFF_KEYS: &[&str] = &["old", "new", "at"];

/// Chaves válidas de `--params` em `import`.
pub(crate) const IMPORT_KEYS: &[&str] = &["path", "source", "name"];

/// Parâmetros de `check`.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct CheckParams {
    /// Caminho do `.akrc`.
    pub(crate) path: Option<String>,
    /// Fonte literal.
    pub(crate) source: Option<String>,
    /// Promove warning a erro.
    pub(crate) strict: Option<bool>,
    /// Resolve `doc=` no disco.
    pub(crate) docs: Option<bool>,
}

/// Parâmetros de `fmt`.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct FmtParams {
    /// Caminho do `.akrc`.
    pub(crate) path: Option<String>,
    /// Fonte literal.
    pub(crate) source: Option<String>,
    /// Grava no próprio arquivo.
    pub(crate) write: Option<bool>,
    /// Sai 1 quando diferir da forma canônica.
    pub(crate) check: Option<bool>,
}

/// Parâmetros de `render`.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct RenderParams {
    /// Caminho do `.akrc`.
    pub(crate) path: Option<String>,
    /// Fonte literal.
    pub(crate) source: Option<String>,
    /// Formato de saída.
    pub(crate) format: Option<String>,
    /// Arquivo de saída.
    pub(crate) out: Option<String>,
    /// Escala da rasterização.
    pub(crate) scale: Option<f32>,
    /// Revisão do Git a ler.
    pub(crate) at: Option<String>,
    /// Valida e reporta sem gravar arquivo.
    pub(crate) dry_run: Option<bool>,
}

/// Parâmetros de `diff`.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct DiffParams {
    /// Caminho do `.akrc` antigo.
    pub(crate) old: Option<String>,
    /// Caminho do `.akrc` novo.
    pub(crate) new: Option<String>,
    /// Revisão do Git a comparar com a árvore de trabalho.
    pub(crate) at: Option<String>,
}

/// Parâmetros de `import`.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ImportParams {
    /// Caminho do `.excalidraw`.
    pub(crate) path: Option<String>,
    /// JSON literal.
    pub(crate) source: Option<String>,
    /// Nome usado no título do `chart` gerado.
    pub(crate) name: Option<String>,
}

/// Desserializa e valida um objeto `--params`.
///
/// # Errors
/// `InvalidInput` quando o JSON não for um objeto, tiver chave desconhecida ou tipo inválido.
pub(crate) fn parse<T>(verb: &str, raw: &str, allowed: &[&str]) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let value: Value = serde_json::from_str(raw)
        .map_err(|error| Error::invalid_input(format!("--params: JSON inválido: {error}")))?;
    let Some(map) = value.as_object() else {
        return Err(Error::invalid_input(
            "--params: espera um objeto JSON, como {\"source\":\"chart v1\"}",
        ));
    };
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(Error::invalid_input(format!(
                "--params: chave desconhecida `{key}` em `{verb}`; válidas: {}",
                allowed.join(", ")
            )));
        }
    }
    serde_json::from_value(value)
        .map_err(|error| Error::invalid_input(format!("--params: {error}")))
}
