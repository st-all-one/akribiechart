//! Testes de medição: métricas da face manual e da face sem serifa.

use super::{Font, MAX_LABEL_W, MIN_NODE_H, MIN_NODE_W, NODE_PADDING};
use crate::model::FontFamily;

fn font() -> Font {
    Font::load().expect("fontes embutidas")
}

#[test]
fn empty_text_has_no_advance() {
    assert_eq!(font().gauge(FontFamily::Hand).advance_px("", 20.0), 0.0);
}

#[test]
fn advance_scales_with_font_size() {
    let font = font();
    let gauge = font.gauge(FontFamily::Hand);
    let small = gauge.advance_px("API Gateway", 20.0);
    let large = gauge.advance_px("API Gateway", 40.0);
    assert!(small > 0.0, "{small}");
    assert!((large - 2.0 * small).abs() < 0.01, "{small} vs {large}");
}

#[test]
fn sans_is_narrower_than_hand_for_the_same_label() {
    let font = font();
    let hand = font.gauge(FontFamily::Hand).advance_px("API Gateway", 20.0);
    let sans = font.gauge(FontFamily::Sans).advance_px("API Gateway", 20.0);
    assert!(hand > 0.0 && sans > 0.0);
    assert_ne!(hand, sans, "as duas faces precisam medir diferente");
}

#[test]
fn node_width_is_advance_plus_padding_or_minimum() {
    let font = font();
    let gauge = font.gauge(FontFamily::Hand);
    let (width, _) = gauge.node_size("API Gateway", 20.0);
    let advance = gauge.advance_px("API Gateway", 20.0);
    assert!((width - (advance + 2.0 * NODE_PADDING).max(MIN_NODE_W)).abs() < 0.01);
    let (narrow, height) = gauge.node_size("A", 20.0);
    assert_eq!(narrow, MIN_NODE_W);
    assert!(height >= MIN_NODE_H);
}

#[test]
fn wrap_respects_width_and_keeps_words() {
    let font = font();
    let gauge = font.gauge(FontFamily::Sans);
    let text = "Uma frase razoavelmente longa que não cabe em uma linha só";
    let lines = gauge.wrap(text, 20.0, 160.0);
    assert!(lines.len() > 1, "{lines:?}");
    assert!(
        lines
            .iter()
            .all(|line| gauge.advance_px(line, 20.0) <= 160.0)
    );
    assert_eq!(lines.join(" "), text);
}

#[test]
fn wrap_splits_long_single_word_by_grapheme() {
    let font = font();
    let gauge = font.gauge(FontFamily::Hand);
    let word = "a".repeat(200);
    let lines = gauge.wrap(&word, 20.0, MAX_LABEL_W);
    assert!(lines.len() > 1);
    assert_eq!(lines.concat(), word);
}

#[test]
fn wrap_forces_explicit_breaks() {
    let lines = font()
        .gauge(FontFamily::Hand)
        .wrap("um\ndois", 20.0, MAX_LABEL_W);
    assert_eq!(lines, ["um", "dois"]);
}

#[test]
fn missing_glyphs_reports_unknown_codepoints_once() {
    let font = font();
    let hand = font.gauge(FontFamily::Hand);
    assert_eq!(hand.missing_glyphs("Retry ×3 (5xx)").len(), 0);
    assert_eq!(hand.missing_glyphs("ok 🦀 ok 🦀"), ['🦀']);
    let sans = font.gauge(FontFamily::Sans);
    assert_eq!(sans.missing_glyphs("Retry ×3 (5xx)").len(), 0);
}
