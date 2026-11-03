//! `akr check`: valida a fonte e reporta diagnósticos posicionados.

pub(crate) mod batch;

use akribie_core::config::Config;
use akribie_core::diag::{self, Diagnostic, has_errors};
use akribie_core::doc::check as doc_check;
use akribie_core::error::Result;
use akribie_core::pipeline;
use akribie_core::ports::Fs;
use akribie_core::text::Font;

use crate::cli::CheckArgs;
use crate::input::{self, Input};
use crate::params::{self, CheckParams};

/// Resultado de uma validação, pronto para texto ou `--json`.
#[derive(Debug)]
pub(crate) struct CheckOutcome {
    /// Rótulo do arquivo (caminho ou `<stdin>`).
    pub(crate) file: String,
    /// Diagnósticos ordenados por posição.
    pub(crate) diagnostics: Vec<Diagnostic>,
    /// `true` quando `--strict` promove avisos a erro.
    pub(crate) strict: bool,
}

impl CheckOutcome {
    /// `true` quando há erro (exit 8), incluindo aviso sob `--strict`.
    pub(crate) fn failed(&self) -> bool {
        has_errors(&self.diagnostics) || (self.strict && !self.diagnostics.is_empty())
    }

    /// Quantidade de diagnósticos de erro.
    pub(crate) fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.is_error())
            .count()
    }

    /// Quantidade de avisos.
    pub(crate) fn warning_count(&self) -> usize {
        self.diagnostics.len().saturating_sub(self.error_count())
    }

    /// Resumo de uma linha (`3 erro(s), 1 aviso(s)`).
    pub(crate) fn summary(&self) -> String {
        format!(
            "{} erro(s), {} aviso(s)",
            self.error_count(),
            self.warning_count()
        )
    }
}

/// Executa a validação.
///
/// # Errors
/// `InvalidInput` no uso inválido de flags/`--params`; `NotFound`/`Io` na leitura do fonte.
pub(crate) fn execute(args: &CheckArgs, fs: &dyn Fs) -> Result<CheckOutcome> {
    if args.params.is_some() && (args.file.is_some() || args.strict) {
        return Err(akribie_core::Error::invalid_input(
            "`--params` substitui posicionais e flags do verbo; use um ou outro (só `--json` soma)",
        ));
    }
    let (input, params) = resolve(args)?;
    let strict = args.strict || params.strict.unwrap_or(false);
    let docs = args.docs || params.docs.unwrap_or(false);
    let file = input.label();
    let source = input.read(fs)?;
    let config = Config::discover(fs, input.path())?;
    let font = Font::load()?;
    let mut report = pipeline::check(&source, &config, &font);
    if docs {
        report
            .diagnostics
            .extend(doc_check(&report.chart, input.path(), fs));
        diag::sort(&mut report.diagnostics);
    }
    Ok(CheckOutcome {
        file,
        diagnostics: report.diagnostics,
        strict,
    })
}

/// Resolve a origem e os parâmetros do verbo.
fn resolve(args: &CheckArgs) -> Result<(Input, CheckParams)> {
    match &args.params {
        Some(raw) => {
            let text = input::read_params(raw)?;
            let params: CheckParams = params::parse("check", &text, params::CHECK_KEYS)?;
            let input = input::from_params(params.path.clone(), params.source.clone())?;
            Ok((input, params))
        }
        None => Ok((
            input::from_positional(args.file.as_deref())?,
            CheckParams::default(),
        )),
    }
}
