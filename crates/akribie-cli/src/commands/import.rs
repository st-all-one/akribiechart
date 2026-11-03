//! `akr import`: `.excalidraw` → `.akrc` (ADR-0004).
//!
//! Perdas são anunciadas (`W005`/`W006`) e a fonte resultante é a forma canônica.

use akribie_core::diag::Diagnostic;
use akribie_core::error::{Error, Result};
use akribie_core::import::import as import_chart;
use akribie_core::ports::Fs;

use crate::cli::ImportArgs;
use crate::input::{self, Input};
use crate::params::{self, ImportParams};

/// Resultado de `akr import`.
#[derive(Debug)]
pub(crate) struct ImportOutcome {
    /// Rótulo da origem.
    pub(crate) file: String,
    /// Fonte `.akrc` canônica.
    pub(crate) source: String,
    /// Perdas anunciadas.
    pub(crate) diagnostics: Vec<Diagnostic>,
}

/// Converte o `.excalidraw` recebido.
///
/// # Errors
/// `NotFound`/`Io` na leitura; `InvalidInput` quando o JSON não for um `.excalidraw`.
pub(crate) fn execute(args: &ImportArgs, fs: &dyn Fs) -> Result<ImportOutcome> {
    if args.params.is_some() && args.file.is_some() {
        return Err(Error::invalid_input(
            "`--params` substitui o posicional; use um ou outro",
        ));
    }
    let (input, name) = if let Some(raw) = &args.params {
        let text = input::read_params(raw)?;
        let params: ImportParams = params::parse("import", &text, params::IMPORT_KEYS)?;
        let input = input::from_params(params.path.clone(), params.source.clone())?;
        let name = params.name.clone().unwrap_or_else(|| name_of(&input));
        (input, name)
    } else {
        let input = input::from_positional(args.file.as_deref())?;
        let name = name_of(&input);
        (input, name)
    };
    let file = input.label();
    let raw = input.read(fs)?;
    let imported = import_chart(&raw, &name)?;
    Ok(ImportOutcome {
        file,
        source: imported.source,
        diagnostics: imported.diagnostics,
    })
}

fn name_of(input: &Input) -> String {
    input
        .path()
        .and_then(|path| path.file_stem())
        .and_then(|stem| stem.to_str())
        .unwrap_or("diagrama")
        .to_string()
}
