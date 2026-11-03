//! `akr diff`: mudanças semânticas entre dois `.akrc`.
//!
//! Compara o modelo resolvido, não a geometria (ADR-0002). Fonte inválida de qualquer lado não
//! produz diff: os diagnósticos daquele arquivo são a saída e o exit é 8.

use akribie_core::Change;
use akribie_core::config::Config;
use akribie_core::diag::{Diagnostic, has_errors};
use akribie_core::error::{Error, Result};
use akribie_core::pipeline;
use akribie_core::ports::{Fs, Git};
use akribie_core::text::Font;

use crate::cli::DiffArgs;
use crate::input::{self, Input};
use crate::params::{self, DiffParams};
use std::path::Path;

/// Resultado de `akr diff`.
#[derive(Debug)]
pub(crate) struct DiffOutcome {
    /// Rótulo do arquivo antigo.
    pub(crate) old: String,
    /// Rótulo do arquivo novo.
    pub(crate) new: String,
    /// Mudanças ordenadas (`added`, `removed`, `changed`).
    pub(crate) changes: Vec<Change>,
    /// Arquivo que impediu o diff, quando houve erro de schema.
    pub(crate) failed_file: Option<String>,
    /// Diagnósticos do arquivo que falhou.
    pub(crate) diagnostics: Vec<Diagnostic>,
    /// Avisos dos dois lados, já com o rótulo do arquivo de origem.
    pub(crate) warnings: Vec<(String, Diagnostic)>,
}

impl DiffOutcome {
    /// `true` quando algum dos lados tem erro e nada foi comparado.
    pub(crate) fn failed(&self) -> bool {
        self.failed_file.is_some()
    }
}

/// Compara dois documentos.
///
/// # Errors
/// `InvalidInput` quando os posicionais estão incompletos ou `--params` é inválido; `NotFound`/`Io`
/// na leitura.
pub(crate) fn execute(args: &DiffArgs, fs: &dyn Fs, git: &dyn Git) -> Result<DiffOutcome> {
    if args.params.is_some() && (args.old.is_some() || args.new.is_some() || args.at.is_some()) {
        return Err(Error::invalid_input(
            "`--params` substitui posicionais e flags do verbo; use um ou outro",
        ));
    }
    let (old, new, at) = match &args.params {
        Some(raw) => {
            let text = input::read_params(raw)?;
            let params: DiffParams = params::parse("diff", &text, params::DIFF_KEYS)?;
            if params.old.is_none() && params.new.is_none() {
                return Err(Error::invalid_input(
                    "--params: informe `old` e `new` (e, opcionalmente, `at`)",
                ));
            }
            (params.old, params.new, params.at)
        }
        None => (args.old.clone(), args.new.clone(), args.at.clone()),
    };
    if let Some(rev) = at {
        return against_revision(old.as_deref(), new.as_deref(), &rev, fs, git);
    }
    let (old_input, new_input) = match (old.as_deref(), new.as_deref()) {
        (Some(old), Some(new)) => pair_of(old, new)?,
        _ => {
            return Err(Error::invalid_input(
                "`diff` precisa de dois arquivos: `akr diff OLD NEW`",
            ));
        }
    };
    let old_label = old_input.label();
    let new_label = new_input.label();
    let config = Config::discover(fs, new_input.path())?;
    let font = Font::load()?;

    Ok(compare_sources(
        &old_input.read(fs)?,
        &new_input.read(fs)?,
        &old_label,
        &new_label,
        &config,
        &font,
    ))
}

/// `akr diff FILE --at REV`: compara a revisão com a árvore de trabalho.
fn against_revision(
    old: Option<&str>,
    new: Option<&str>,
    rev: &str,
    fs: &dyn Fs,
    git: &dyn Git,
) -> Result<DiffOutcome> {
    let file = match (old, new) {
        (None, Some(file)) | (Some(file), None) => file,
        (Some(_), Some(_)) => {
            return Err(Error::invalid_input(
                "`--at REV` compara um arquivo com a revisão dele; informe só FILE",
            ));
        }
        (None, None) => return Err(Error::invalid_input("`--at REV` exige FILE")),
    };
    let path = Path::new(file);
    let old = git.show(rev, path)?;
    let new = fs.read(path)?;
    let config = Config::discover(fs, Some(path))?;
    let font = Font::load()?;
    Ok(compare_sources(
        &old,
        &new,
        &format!("{file}@{rev}"),
        file,
        &config,
        &font,
    ))
}

fn compare_sources(
    old_source: &str,
    new_source: &str,
    old: &str,
    new: &str,
    config: &Config,
    font: &Font,
) -> DiffOutcome {
    let old_report = pipeline::check(old_source, config, font);
    if has_errors(&old_report.diagnostics) {
        return failed(old, old_report.diagnostics);
    }
    let new_report = pipeline::check(new_source, config, font);
    if has_errors(&new_report.diagnostics) {
        return failed(new, new_report.diagnostics);
    }
    let mut warnings: Vec<(String, Diagnostic)> = old_report
        .diagnostics
        .into_iter()
        .map(|diagnostic| (old.to_string(), diagnostic))
        .collect();
    warnings.extend(
        new_report
            .diagnostics
            .into_iter()
            .map(|diagnostic| (new.to_string(), diagnostic)),
    );
    DiffOutcome {
        old: old.to_string(),
        new: new.to_string(),
        changes: akribie_core::diff(&old_report.chart, &new_report.chart),
        failed_file: None,
        diagnostics: Vec::new(),
        warnings,
    }
}

fn failed(file: &str, diagnostics: Vec<Diagnostic>) -> DiffOutcome {
    DiffOutcome {
        old: file.to_string(),
        new: file.to_string(),
        changes: Vec::new(),
        failed_file: Some(file.to_string()),
        diagnostics,
        warnings: Vec::new(),
    }
}

fn pair_of(old: &str, new: &str) -> Result<(Input, Input)> {
    if old == "-" && new == "-" {
        return Err(Error::invalid_input(
            "`diff - -`: a entrada padrão só pode ser lida uma vez",
        ));
    }
    Ok((
        input::from_positional(Some(old))?,
        input::from_positional(Some(new))?,
    ))
}
