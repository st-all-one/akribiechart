//! Medição de texto com as fontes embutidas (Excalifont e Liberation Sans, ambas OFL).
//!
//! A mesma medição alimenta layout, pintura e SVG: se o número muda aqui, muda em todo lugar. Por
//! isso as fontes são **embutidas** e versionadas (`assets/FONT.lock`), e o shaping usa o Hárfbuzz
//! portátil do `harfrust` — métricas idênticas em qualquer máquina.
//!
//! `font=sans` (padrão) mede e desenha com a Liberation Sans; `font=hand` (opcional) usa a
//! Excalifont. As duas faces vivem no mesmo [`Font`]: `check` audita glifos da família escolhida e a
//! rasterização registra ambas no `usvg`.

use harfrust::{Buffer, Font as Face, ShapeOptions, ShaperFont};
use unicode_segmentation::UnicodeSegmentation;

use crate::error::{Error, Result};
use crate::model::FontFamily;

/// Largura mínima de um nó.
pub const MIN_NODE_W: f32 = 96.0;
/// Altura mínima de um nó.
pub const MIN_NODE_H: f32 = 44.0;
/// Respiro entre o rótulo e a borda.
pub const NODE_PADDING: f32 = 12.0;
/// Entrelinha, como múltiplo do tamanho da fonte.
pub const LINE_HEIGHT: f32 = 1.35;
/// Largura máxima do bloco de rótulo antes de quebrar linha.
pub const MAX_LABEL_W: f32 = 320.0;

/// Bytes da fonte manual (usados também pela rasterização).
pub const FONT_BYTES: &[u8] = include_bytes!("../../../assets/Excalifont-Regular.ttf");
/// Bytes da fonte sem serifa (usados também pela rasterização).
pub const SANS_BYTES: &[u8] = include_bytes!("../../../assets/LiberationSans-Regular.ttf");

/// Pilha tipográfica do SVG para `font=hand` (a fonte embutida vem primeiro).
pub const HAND_STACK: &str = "Excalifont, Virgil, Comic Shanns, cursive";
/// Pilha tipográfica do SVG para `font=sans`.
pub const SANS_STACK: &str = "Liberation Sans, Arial, Helvetica, sans-serif";

/// Pilha tipográfica da família declarada.
pub const fn stack(family: FontFamily) -> &'static str {
    match family {
        FontFamily::Hand => HAND_STACK,
        FontFamily::Sans => SANS_STACK,
    }
}

/// Uma face medível: shaping e métricas de uma única fonte.
pub struct Gauge {
    face: Face,
    upem: f32,
}

impl Gauge {
    fn new(bytes: &'static [u8], asset: &str) -> Result<Self> {
        let face = Face::new(bytes, 0)
            .ok_or_else(|| Error::internal(format!("fonte embutida inválida ({asset})")))?;
        let upem = to_px(i32::from(face.units_per_em()));
        Ok(Self { face, upem })
    }

    /// Avanço horizontal de `text` em pixels, no tamanho `size`.
    pub fn advance_px(&self, text: &str, size: f32) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        let shaper = ShaperFont::new(&self.face);
        let mut buffer = Buffer::new();
        buffer.push_str(text);
        // Sem direção/script o `shape` devolve `DirectionUnset` e a medição cairia no fallback
        // por contagem de caracteres — que ignora os avanços reais da fonte.
        buffer.guess_segment_properties();
        if harfrust::shape(&shaper, &mut buffer, ShapeOptions::default()).is_err() {
            // Com a própria fonte embutida o shaping não falha; se falhar, cai no pior caso
            // (uma estimativa por grafema) em vez de mentir que a largura é zero.
            let count = u16::try_from(text.chars().count()).unwrap_or(u16::MAX);
            return f32::from(count) * size * 0.5;
        }
        let units: i32 = buffer
            .glyph_positions()
            .iter()
            .map(|pos| pos.x_advance)
            .sum();
        to_px(units) * size / self.upem
    }

    /// Quebra `text` em linhas que cabem em `max_px`.
    ///
    /// Quebra em espaço; palavra maior que a linha é partida por grafemas (nunca por byte).
    /// `\n` no texto força linha nova.
    pub fn wrap(&self, text: &str, size: f32, max_px: f32) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        for paragraph in text.split('\n') {
            self.wrap_paragraph(paragraph, size, max_px, &mut lines);
        }
        lines
    }

    fn wrap_paragraph(&self, paragraph: &str, size: f32, max_px: f32, lines: &mut Vec<String>) {
        let mut line = String::new();
        for word in paragraph.split(' ').filter(|word| !word.is_empty()) {
            self.place_word(word, size, max_px, &mut line, lines);
        }
        lines.push(line);
    }

    fn place_word(
        &self,
        word: &str,
        size: f32,
        max_px: f32,
        line: &mut String,
        lines: &mut Vec<String>,
    ) {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if self.advance_px(&candidate, size) <= max_px {
            *line = candidate;
            return;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(line));
        }
        if self.advance_px(word, size) <= max_px {
            *line = word.to_string();
            return;
        }
        *line = self.break_word(word, size, max_px, lines);
    }

    /// Parte uma palavra maior que a linha em pedaços por grafema; devolve o resto.
    fn break_word(&self, word: &str, size: f32, max_px: f32, lines: &mut Vec<String>) -> String {
        let mut chunk = String::new();
        for grapheme in word.graphemes(true) {
            let candidate = format!("{chunk}{grapheme}");
            if !chunk.is_empty() && self.advance_px(&candidate, size) > max_px {
                lines.push(std::mem::take(&mut chunk));
            }
            chunk.push_str(grapheme);
        }
        chunk
    }

    /// Codepoints distintos de `text` sem glifo na fonte.
    pub fn missing_glyphs(&self, text: &str) -> Vec<char> {
        let mut missing: Vec<char> = Vec::new();
        for character in text.chars() {
            if character.is_whitespace() || character.is_control() {
                continue;
            }
            let glyph = self.face.charmap().map_unicode(u32::from(character));
            if glyph.is_none() && !missing.contains(&character) {
                missing.push(character);
            }
        }
        missing
    }

    /// Tamanho do rótulo já quebrado: largura da linha mais larga e altura total.
    pub fn label_size(&self, text: &str, size: f32) -> (f32, f32) {
        let lines = self.wrap(text, size, MAX_LABEL_W);
        let width = lines
            .iter()
            .map(|line| self.advance_px(line, size))
            .fold(0.0_f32, f32::max);
        let count = u16::try_from(lines.len().max(1)).unwrap_or(u16::MAX);
        (width, size * LINE_HEIGHT * f32::from(count))
    }

    /// Tamanho de um nó retangular que acomoda o rótulo (mínimos e respiro aplicados).
    pub fn node_size(&self, label: &str, size: f32) -> (f32, f32) {
        let (width, height) = self.label_size(label, size);
        let breath = 2.0 * NODE_PADDING;
        (
            (width + breath).max(MIN_NODE_W),
            (height + breath).max(MIN_NODE_H),
        )
    }
}

/// As duas faces embutidas, escolhidas por `chart.font`.
pub struct Font {
    hand: Gauge,
    sans: Gauge,
}

impl Font {
    /// Carrega as duas fontes embutidas.
    ///
    /// # Errors
    /// `Internal` se algum dos bytes embutidos não formar uma fonte válida (build quebrado).
    pub fn load() -> Result<Self> {
        Ok(Self {
            hand: Gauge::new(FONT_BYTES, "assets/Excalifont-Regular.ttf")?,
            sans: Gauge::new(SANS_BYTES, "assets/LiberationSans-Regular.ttf")?,
        })
    }

    /// Face da família declarada.
    pub fn gauge(&self, family: FontFamily) -> &Gauge {
        match family {
            FontFamily::Hand => &self.hand,
            FontFamily::Sans => &self.sans,
        }
    }
}

/// Unidades de fonte (i32) → pixels (f32), sem `as` e sem saturação silenciosa.
fn to_px(units: i32) -> f32 {
    let clamped = units.clamp(i32::from(i16::MIN), i32::from(i16::MAX));
    f32::from(i16::try_from(clamped).unwrap_or(0))
}

#[cfg(test)]
mod tests;
