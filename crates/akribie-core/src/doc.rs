//! Linhagem documental: `chart ... doc=path#anchor` (ADR-0002).
//!
//! `doc=` é metadado puro: não muda layout nem desenho. Só é **resolvido** com `--docs`, porque
//! tocar o disco em todo `check` custaria caro e não é necessário para compilar.

use std::path::{Path, PathBuf};

use crate::diag::{Diagnostic, W004};
use crate::model::Chart;
use crate::ports::Fs;
use crate::span::Span;

/// Verifica se o alvo de `doc=` existe (arquivo e âncora).
///
/// O caminho é relativo ao diretório do `.akrc` (`base`). A âncora aceita as duas formas que um
/// leitor humano aceita: o texto literal (sem caixa) ou o *slug* do título (`# Guia Rápida` →
/// `guia-rapida`), que é o que o GitHub gera.
pub fn check(chart: &Chart, base: Option<&Path>, fs: &dyn Fs) -> Vec<Diagnostic> {
    let Some(doc) = chart.doc.as_deref() else {
        return Vec::new();
    };
    let (path, anchor) = match doc.split_once('#') {
        Some((path, anchor)) => (path, Some(anchor)),
        None => (doc, None),
    };
    let root = base.and_then(Path::parent);
    let target: PathBuf = match root {
        Some(root) => root.join(path),
        None => PathBuf::from(path),
    };
    if !fs.exists(&target) {
        return vec![missing(format!(
            "documento `{}` não encontrado (de `{doc}`)",
            target.display()
        ))];
    }
    let Some(anchor) = anchor.filter(|anchor| !anchor.is_empty()) else {
        return Vec::new();
    };
    match fs.read(&target) {
        Ok(text) if anchor_present(&text, anchor) => Vec::new(),
        Ok(_) => vec![missing(format!(
            "âncora `#{anchor}` não encontrada em `{}`",
            target.display()
        ))],
        Err(error) => vec![missing(format!(
            "não foi possível ler `{}`: {}",
            target.display(),
            error.message()
        ))],
    }
}

/// `true` quando a âncora aparece como texto (sem caixa) ou como slug de algum título.
fn anchor_present(text: &str, anchor: &str) -> bool {
    let wanted = anchor.to_lowercase();
    if text.to_lowercase().contains(&wanted) {
        return true;
    }
    text.lines()
        .filter(|line| line.trim_start().starts_with('#'))
        .any(|line| slug(line) == wanted)
}

/// Slug no estilo GitHub: caixa baixa, espaços e pontuação viram `-`, sem hífen nas pontas.
fn slug(line: &str) -> String {
    let heading = line
        .trim_start()
        .trim_start_matches('#')
        .trim()
        .to_lowercase();
    let mut slug = String::with_capacity(heading.len());
    for character in heading.chars() {
        if character.is_alphanumeric() {
            slug.push(character);
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_string()
}

fn missing(message: String) -> Diagnostic {
    Diagnostic::new(W004, Span::new(1, 1), message).with_hint(
        "corrija `doc=` (caminho relativo ao `.akrc`, âncora depois de `#`) ou remova a chave",
    )
}
