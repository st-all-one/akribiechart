//! `render --batch`: um item JSONL por linha, best-effort, na ordem do arquivo (§2.3).
//!
//! Item = chaves de `render --params` + `name`. `format` cai para `svg`, `out` cai para
//! `<out-dir>/<name>.<ext>`, e `--dry-run` calcula bytes sem gravar nada.

use std::path::Path;

use akribie_core::config::Config;
use akribie_core::diag::{self, Diagnostic, has_errors};
use akribie_core::emit::{encode, excalidraw, scene_json, svg};
use akribie_core::error::{Error, Result};
use akribie_core::layout;
use akribie_core::paint;
use akribie_core::pipeline;
use akribie_core::ports::{Fs, Git};
use akribie_core::text::Font;

use crate::batch::{self, ItemError};
use crate::cli::RenderArgs;
use crate::commands::format::Format;
use crate::input;
use crate::params::{self, RenderParams};

/// Resultado de um item do lote.
#[derive(Debug)]
pub(crate) struct BatchRenderOutcome {
    /// Posição no lote (0-based).
    pub(crate) index: usize,
    /// Nome do item.
    pub(crate) name: String,
    /// Alvo gravado; em `--dry-run`, o alvo que seria gravado.
    pub(crate) out: Option<String>,
    /// Bytes do artefato.
    pub(crate) bytes: Option<usize>,
    /// Diagnósticos do item (avisos incluídos).
    pub(crate) diagnostics: Vec<Diagnostic>,
    /// Falha que impede o item de contar como sucesso.
    pub(crate) error: Option<ItemError>,
}

impl BatchRenderOutcome {
    /// `true` quando o item renderizou sem erro nem falha de leitura.
    pub(crate) fn ok(&self) -> bool {
        self.error.is_none() && !has_errors(&self.diagnostics)
    }

    /// Alvo para a linha de texto: caminho, ou o nome quando não houve alvo.
    pub(crate) fn target(&self) -> &str {
        self.out.as_deref().unwrap_or(&self.name)
    }

    /// Código e mensagem para a saída de erro (`schema` quando há diagnóstico de erro).
    pub(crate) fn failure(&self) -> Option<(&'static str, String)> {
        if let Some(error) = &self.error {
            return Some((error.code, error.message.clone()));
        }
        self.diagnostics
            .iter()
            .find(|diagnostic| diagnostic.is_error())
            .map(|diagnostic| (diagnostic.code.code, diagnostic.message.clone()))
    }
}

/// Executa `render --batch`.
///
/// # Errors
/// `InvalidInput` com `--params`/`FILE`, lote vazio ou uso inválido; `NotFound` quando `--out-dir`
/// não existir; `Io` na leitura do lote.
pub(crate) fn execute(
    args: &RenderArgs,
    fs: &dyn Fs,
    git: &dyn Git,
) -> Result<Vec<BatchRenderOutcome>> {
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
    let out_dir = args.out_dir.clone().unwrap_or_else(|| ".".to_string());
    if !fs.exists(Path::new(&out_dir)) {
        return Err(Error::not_found(format!(
            "diretório de saída não encontrado: `{out_dir}`"
        )));
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
        .map(|item| run(item, args, &out_dir, fs, git, &font))
        .collect())
}

fn run(
    item: batch::Item,
    args: &RenderArgs,
    out_dir: &str,
    fs: &dyn Fs,
    git: &dyn Git,
    font: &Font,
) -> BatchRenderOutcome {
    if let Some(error) = item.error.clone() {
        return failed(item, error);
    }
    let params: RenderParams = match params::parse("render", &item.params, params::RENDER_KEYS) {
        Ok(params) => params,
        Err(error) => return failed(item, ItemError::from_error(&error)),
    };
    let path = params.path.clone();
    let source = match (&params.at, &params.path, &params.source) {
        (_, Some(_), Some(_)) => {
            return failed(
                item,
                ItemError::invalid("informe `path` ou `source`, não ambos"),
            );
        }
        (_, None, None) => {
            return failed(item, ItemError::invalid("informe `path` ou `source`"));
        }
        (Some(rev), Some(path), None) => match git.show(rev, Path::new(path)) {
            Ok(source) => source,
            Err(error) => return failed(item, ItemError::from_error(&error)),
        },
        (Some(_), None, Some(_)) => {
            return failed(
                item,
                ItemError::invalid("`at` exige `path`: revisão do Git é de arquivo"),
            );
        }
        (None, Some(path), None) => match fs.read(Path::new(path)) {
            Ok(source) => source,
            Err(error) => return failed(item, ItemError::from_error(&error)),
        },
        (None, None, Some(source)) => source.clone(),
    };
    let format = match Format::parse(
        params
            .format
            .as_deref()
            .or(args.format.as_deref())
            .unwrap_or("svg"),
    ) {
        Ok(format) => format,
        Err(error) => return failed(item, ItemError::from_error(&error)),
    };
    let scale = params.scale.or(args.scale).unwrap_or(2.0);
    let dry_run = params.dry_run.unwrap_or(args.dry_run);
    let out = params.out.clone().unwrap_or_else(|| {
        Path::new(out_dir)
            .join(format!("{}.{}", item.name, format.extension()))
            .display()
            .to_string()
    });
    let config = match Config::discover(fs, path.as_deref().map(Path::new)) {
        Ok(config) => config,
        Err(error) => return failed(item, ItemError::from_error(&error)),
    };
    let report = pipeline::check(&source, &config, font);
    if has_errors(&report.diagnostics) {
        let mut outcome = failed(
            item,
            ItemError {
                code: "schema",
                message: first_error(&report.diagnostics),
            },
        );
        outcome.diagnostics = report.diagnostics;
        return outcome;
    }
    let placed = layout::layout(&report.chart, font);
    let document = match format {
        Format::Scene => scene_json(&placed, &report.chart),
        Format::Excalidraw => excalidraw(&report.chart, &placed),
        Format::Svg | Format::Raster(_) => {
            let scene = paint::paint(&report.chart, &placed, font);
            svg(&scene, report.chart.theme)
        }
    };
    let bytes = match format {
        Format::Raster(raster) => match encode(&document, raster, scale) {
            Ok(bytes) => bytes,
            Err(error) => return failed(item, ItemError::from_error(&error)),
        },
        Format::Scene | Format::Excalidraw | Format::Svg => document.into_bytes(),
    };
    if !dry_run && let Err(error) = fs.write(Path::new(&out), &bytes) {
        return failed(item, ItemError::from_error(&error));
    }
    let mut diagnostics = report.diagnostics;
    diag::sort(&mut diagnostics);
    BatchRenderOutcome {
        index: item.index,
        name: item.name,
        out: Some(out),
        bytes: Some(bytes.len()),
        diagnostics,
        error: None,
    }
}

/// Item que não chegou a renderizar: sem alvo e sem bytes.
fn failed(item: batch::Item, error: ItemError) -> BatchRenderOutcome {
    BatchRenderOutcome {
        index: item.index,
        name: item.name,
        out: None,
        bytes: None,
        diagnostics: Vec::new(),
        error: Some(error),
    }
}

/// Primeira linha de erro, no formato `E003 invalid-value: …`.
fn first_error(diagnostics: &[Diagnostic]) -> String {
    diagnostics
        .iter()
        .find(|diagnostic| diagnostic.is_error())
        .map_or_else(
            || "fonte inválida".to_string(),
            |diagnostic| {
                format!(
                    "{} {}: {}",
                    diagnostic.code.code, diagnostic.code.name, diagnostic.message
                )
            },
        )
}
