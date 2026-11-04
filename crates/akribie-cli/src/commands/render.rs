//! `akr render`: compila o `.akrc` em SVG (`stdout`) ou raster (`--out`).
//!
//! O verbo não renderiza nada inválido: se `check` encontrar erro, os diagnósticos são a saída e o
//! exit é 8. Warnings não bloqueiam. `--dry-run` calcula bytes e dimensões sem gravar arquivo.

pub(crate) mod batch;

use std::path::Path;

use akribie_core::config::Config;
use akribie_core::diag::{self, Diagnostic, has_errors};
use akribie_core::emit::{encode, excalidraw, scene_json, svg};
use akribie_core::error::{Error, Result};
use akribie_core::layout::{self, Placed};
use akribie_core::paint;
use akribie_core::pipeline;
use akribie_core::ports::{Fs, Git};
use akribie_core::text::Font;

use crate::cli::RenderArgs;
use crate::commands::format::Format;
use crate::input::{self, Input};
use crate::params::{self, RenderParams};

/// Se esta execução grava arquivos (`--dry-run` desliga).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Writing {
    /// Grava o artefato no destino.
    Yes,
    /// Apenas calcula; nada é escrito.
    No,
}

/// Resultado de `akr render`.
#[derive(Debug)]
pub(crate) struct RenderOutcome {
    /// Rótulo do arquivo de entrada.
    pub(crate) file: String,
    /// Formato pedido.
    pub(crate) format: &'static str,
    /// Conteúdo, quando a renderização foi para `stdout`.
    pub(crate) payload: Option<String>,
    /// Arquivo gravado; em `--dry-run`, o arquivo que seria gravado.
    pub(crate) written: Option<String>,
    /// Bytes da saída (documento em memória ou arquivo raster).
    pub(crate) bytes: Option<usize>,
    /// Largura do canvas (pixels no raster, pontos nos vetoriais).
    pub(crate) width: f64,
    /// Altura do canvas (pixels no raster, pontos nos vetoriais).
    pub(crate) height: f64,
    /// Diagnósticos (vazios quando tudo certo).
    pub(crate) diagnostics: Vec<Diagnostic>,
    /// `true` quando há erro de schema (nada foi renderizado).
    pub(crate) failed: bool,
    /// `true` quando `--dry-run` suprimiu a escrita.
    pub(crate) dry_run: bool,
}

/// Executa a renderização.
///
/// # Errors
/// `InvalidInput` em flags/`--params`; `NotFound`/`Io` na leitura ou escrita; `Internal` se a
/// rasterização falhar.
pub(crate) fn execute(args: &RenderArgs, fs: &dyn Fs, git: &dyn Git) -> Result<RenderOutcome> {
    if args.params.is_some()
        && (args.file.is_some()
            || args.format.is_some()
            || args.out.is_some()
            || args.scale.is_some()
            || args.at.is_some()
            || args.dry_run)
    {
        return Err(Error::invalid_input(
            "`--params` substitui posicionais e flags do verbo; use um ou outro",
        ));
    }
    let (input, params) = resolve(args)?;
    let format = Format::parse(
        params
            .format
            .as_deref()
            .or(args.format.as_deref())
            .unwrap_or("svg"),
    )?;
    let out = params.out.clone().or_else(|| args.out.clone());
    let scale = params.scale.or(args.scale).unwrap_or(2.0);
    let dry_run = params.dry_run.unwrap_or(args.dry_run);
    let writing = if dry_run { Writing::No } else { Writing::Yes };
    if matches!(format, Format::Raster(_)) && out.is_none() {
        return Err(Error::invalid_input(
            "`--format png|jpg|avif` exige `--out PATH`: binário não vai para o terminal",
        ));
    }
    let revision = params.at.clone().or_else(|| args.at.clone());
    let (file, source) = match &revision {
        Some(rev) => {
            let path = input.path().ok_or_else(|| {
                Error::invalid_input("`--at REV` exige um arquivo (não `-` nem `source`)")
            })?;
            (format!("{}@{rev}", input.label()), git.show(rev, path)?)
        }
        None => (input.label(), input.read(fs)?),
    };
    let config = Config::discover(fs, input.path())?;
    let font = Font::load()?;
    let report = pipeline::check(&source, &config, &font);
    if has_errors(&report.diagnostics) {
        return Ok(RenderOutcome {
            file,
            format: format.name(),
            payload: None,
            written: None,
            bytes: None,
            width: 0.0,
            height: 0.0,
            diagnostics: report.diagnostics,
            failed: true,
            dry_run,
        });
    }
    let placed = layout::layout(&report.chart, &font);
    let (width, height) = canvas(&placed, format, scale);
    let (payload, written, bytes) = match format {
        Format::Scene => {
            let document = scene_json(&placed, &report.chart);
            text_output(document, out, fs, writing)?
        }
        Format::Excalidraw => {
            let document = excalidraw(&report.chart, &placed);
            text_output(document, out, fs, writing)?
        }
        Format::Svg => {
            let scene = paint::paint(&report.chart, &placed);
            let document = svg(&scene, report.chart.theme);
            text_output(document, out, fs, writing)?
        }
        Format::Raster(raster) => {
            let scene = paint::paint(&report.chart, &placed);
            let document = svg(&scene, report.chart.theme);
            let bytes = encode(&document, raster, scale)?;
            let size = bytes.len();
            let Some(path) = out else {
                return Err(Error::invalid_input("formato raster exige `--out`"));
            };
            if matches!(writing, Writing::Yes) {
                fs.write(Path::new(&path), &bytes)?;
            }
            (None, Some(path), Some(size))
        }
    };
    let mut diagnostics = report.diagnostics;
    diag::sort(&mut diagnostics);
    Ok(RenderOutcome {
        file,
        format: format.name(),
        payload,
        written,
        bytes,
        width,
        height,
        diagnostics,
        failed: false,
        dry_run,
    })
}

/// Dimensões do artefato: pontos no vetorial, pixels no raster (`scale` aplicada).
fn canvas(placed: &Placed, format: Format, scale: f32) -> (f64, f64) {
    let factor = match format {
        Format::Raster(_) => f64::from(scale),
        Format::Scene | Format::Excalidraw | Format::Svg => 1.0,
    };
    (
        round2(f64::from(placed.width) * factor),
        round2(f64::from(placed.height) * factor),
    )
}

/// Duas casas decimais, sem ruído de `f32` no JSON.
fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Texto para `stdout` ou para `--out`, com tamanho em bytes; `dry_run` não grava.
fn text_output(
    document: String,
    out: Option<String>,
    fs: &dyn Fs,
    writing: Writing,
) -> Result<(Option<String>, Option<String>, Option<usize>)> {
    let size = document.len();
    match out {
        Some(path) => {
            if matches!(writing, Writing::Yes) {
                fs.write(Path::new(&path), document.as_bytes())?;
            }
            Ok((None, Some(path), Some(size)))
        }
        None => Ok((Some(document), None, Some(size))),
    }
}

fn resolve(args: &RenderArgs) -> Result<(Input, RenderParams)> {
    match &args.params {
        Some(raw) => {
            let text = input::read_params(raw)?;
            let params: RenderParams = params::parse("render", &text, params::RENDER_KEYS)?;
            let input = input::from_params(params.path.clone(), params.source.clone())?;
            Ok((input, params))
        }
        None => Ok((
            input::from_positional(args.file.as_deref())?,
            RenderParams::default(),
        )),
    }
}
