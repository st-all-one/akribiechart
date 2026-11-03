//! `akr fmt`: normaliza um `.akrc` para a forma canônica.

use akribie_core::diag::{self, Diagnostic, has_errors};
use akribie_core::error::{Error, Result};
use akribie_core::fmt::format;
use akribie_core::parse::parse;
use akribie_core::ports::Fs;
use std::path::Path;

use crate::cli::FmtArgs;
use crate::input::{self, Input};
use crate::params::{self, FmtParams};

/// O que fazer com o resultado da formatação.
pub(crate) enum Action {
    /// Imprimir a forma canônica.
    Emit(String),
    /// Comparar e sair 1 quando diferir.
    Check { formatted: String, original: String },
    /// Já gravado no arquivo.
    Written,
    /// Fonte inválida: apenas reportar diagnósticos.
    Failed,
}

/// Resultado de `akr fmt`.
pub(crate) struct FmtOutcome {
    /// Rótulo do arquivo (caminho ou `<stdin>`).
    pub(crate) file: String,
    /// Diagnósticos de schema (impedem formatar quando são erros).
    pub(crate) diagnostics: Vec<Diagnostic>,
    /// Ação a executar.
    pub(crate) action: Action,
}

/// Executa a formatação.
///
/// # Errors
/// `InvalidInput` em combinação de flags, `--params` com posicional, ou `--write` sem arquivo;
/// `NotFound`/`Io` na leitura ou escrita.
pub(crate) fn execute(args: &FmtArgs, fs: &dyn Fs) -> Result<FmtOutcome> {
    if args.params.is_some() && (args.file.is_some() || args.write || args.check) {
        return Err(Error::invalid_input(
            "`--params` substitui posicionais e flags do verbo; use um ou outro",
        ));
    }
    if args.write && args.check {
        return Err(Error::invalid_input(
            "`--write` e `--check` são mutuamente exclusivos",
        ));
    }

    let (input, params) = resolve(args)?;
    let write = args.write || params.write.unwrap_or(false);
    let check = args.check || params.check.unwrap_or(false);
    if write && check {
        return Err(Error::invalid_input(
            "`--write` e `--check` são mutuamente exclusivos",
        ));
    }

    let file = input.label();
    let original = input.read(fs)?;
    let (document, mut diagnostics) = parse(&original);
    diag::sort(&mut diagnostics);
    if has_errors(&diagnostics) {
        return Ok(FmtOutcome {
            file,
            diagnostics,
            action: Action::Failed,
        });
    }

    let formatted = format(&document);
    let action = if write {
        let Input::Path(path) = &input else {
            return Err(Error::invalid_input(
                "`--write` exige um arquivo; stdin e `source` não têm onde gravar",
            ));
        };
        fs.write(Path::new(path), formatted.as_bytes())?;
        Action::Written
    } else if check {
        Action::Check {
            formatted,
            original,
        }
    } else {
        Action::Emit(formatted)
    };
    Ok(FmtOutcome {
        file,
        diagnostics,
        action,
    })
}

fn resolve(args: &FmtArgs) -> Result<(Input, FmtParams)> {
    match &args.params {
        Some(raw) => {
            let text = input::read_params(raw)?;
            let params: FmtParams = params::parse("fmt", &text, params::FMT_KEYS)?;
            let input = input::from_params(params.path.clone(), params.source.clone())?;
            Ok((input, params))
        }
        None => Ok((
            input::from_positional(args.file.as_deref())?,
            FmtParams::default(),
        )),
    }
}
