//! `check --batch`: valida cada item JSONL, best-effort, na ordem do arquivo (§2.3).
//!
//! Item = chaves de `check --params` (`docs`, `strict`) + `name`. Nenhum arquivo é escrito; a
//! saída conta avisos por item e o processo falha só quando todos falham.

use std::path::Path;

use akribie_core::config::Config;
use akribie_core::diag::{self, Diagnostic, has_errors};
use akribie_core::doc::check as doc_check;
use akribie_core::error::{Error, Result};
use akribie_core::pipeline;
use akribie_core::ports::Fs;
use akribie_core::text::Font;

use crate::batch::{self, ItemError};
use crate::cli::CheckArgs;
use crate::input;
use crate::params::{self, CheckParams};

/// Resultado de um item do lote de validação.
#[derive(Debug)]
pub(crate) struct BatchCheckOutcome {
    /// Posição no lote (0-based).
    pub(crate) index: usize,
    /// Nome do item.
    pub(crate) name: String,
    /// Diagnósticos ordenados por posição.
    pub(crate) diagnostics: Vec<Diagnostic>,
    /// Falha de leitura, JSONL ou origem.
    pub(crate) error: Option<ItemError>,
    /// `--strict` do item promove aviso a erro.
    pub(crate) strict: bool,
}

impl BatchCheckOutcome {
    /// `true` quando o item passou (sem erro; sob `strict`, sem aviso também).
    pub(crate) fn ok(&self) -> bool {
        if self.error.is_some() || has_errors(&self.diagnostics) {
            return false;
        }
        !self.strict || self.diagnostics.is_empty()
    }

    /// Código e mensagem para a saída de erro.
    pub(crate) fn failure(&self) -> Option<(&'static str, String)> {
        if let Some(error) = &self.error {
            return Some((error.code, error.message.clone()));
        }
        self.diagnostics
            .iter()
            .find(|diagnostic| diagnostic.is_error())
            .or_else(|| self.diagnostics.first())
            .map(|diagnostic| (diagnostic.code.code, diagnostic.message.clone()))
    }

    /// Quantidade de avisos do item.
    pub(crate) fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| !diagnostic.is_error())
            .count()
    }
}

/// Executa `check --batch`.
///
/// # Errors
/// `InvalidInput` com `--params`/`FILE` ou lote vazio; `NotFound`/`Io` na leitura do lote.
pub(crate) fn execute(args: &CheckArgs, fs: &dyn Fs) -> Result<Vec<BatchCheckOutcome>> {
    if args.params.is_some() {
        return Err(Error::invalid_input(
            "`--batch` e `--params` são exclusivos: o lote traz os parâmetros por item",
        ));
    }
    if args.file.is_some() {
        return Err(Error::invalid_input(
            "`--batch` lê o lote; não aceita FILE (cada item informa `path` ou `source`)",
        ));
    }
    let batch_path = args.batch.as_deref().unwrap_or("-");
    let text = if batch_path == "-" {
        input::read_stdin()?
    } else {
        fs.read(Path::new(batch_path))?
    };
    let lines = pipeline::batch_lines(&text);
    if lines.is_empty() {
        return Err(Error::invalid_input(
            "lote vazio: nenhuma linha de item em `--batch`",
        ));
    }
    let font = Font::load()?;
    Ok(batch::items(lines)
        .into_iter()
        .map(|item| run(item, fs, &font))
        .collect())
}

fn run(item: batch::Item, fs: &dyn Fs, font: &Font) -> BatchCheckOutcome {
    if let Some(error) = item.error {
        return BatchCheckOutcome {
            index: item.index,
            name: item.name,
            diagnostics: Vec::new(),
            error: Some(error),
            strict: false,
        };
    }
    let params: CheckParams = match params::parse("check", &item.params, params::CHECK_KEYS) {
        Ok(params) => params,
        Err(error) => {
            return BatchCheckOutcome {
                index: item.index,
                name: item.name,
                diagnostics: Vec::new(),
                error: Some(ItemError::from_error(&error)),
                strict: false,
            };
        }
    };
    let strict = params.strict.unwrap_or(false);
    let docs = params.docs.unwrap_or(false);
    let path = params.path.clone();
    let source = match (&params.path, &params.source) {
        (Some(_), Some(_)) => {
            return BatchCheckOutcome {
                index: item.index,
                name: item.name,
                diagnostics: Vec::new(),
                error: Some(ItemError::invalid("informe `path` ou `source`, não ambos")),
                strict,
            };
        }
        (None, None) => {
            return BatchCheckOutcome {
                index: item.index,
                name: item.name,
                diagnostics: Vec::new(),
                error: Some(ItemError::invalid("informe `path` ou `source`")),
                strict,
            };
        }
        (Some(path), None) => match fs.read(Path::new(path)) {
            Ok(source) => source,
            Err(error) => {
                return BatchCheckOutcome {
                    index: item.index,
                    name: item.name,
                    diagnostics: Vec::new(),
                    error: Some(ItemError::from_error(&error)),
                    strict,
                };
            }
        },
        (None, Some(source)) => source.clone(),
    };
    let config = match Config::discover(fs, path.as_deref().map(Path::new)) {
        Ok(config) => config,
        Err(error) => {
            return BatchCheckOutcome {
                index: item.index,
                name: item.name,
                diagnostics: Vec::new(),
                error: Some(ItemError::from_error(&error)),
                strict,
            };
        }
    };
    let mut report = pipeline::check(&source, &config, font);
    if docs {
        report
            .diagnostics
            .extend(doc_check(&report.chart, path.as_deref().map(Path::new), fs));
        diag::sort(&mut report.diagnostics);
    }
    BatchCheckOutcome {
        index: item.index,
        name: item.name,
        diagnostics: report.diagnostics,
        error: None,
        strict,
    }
}
