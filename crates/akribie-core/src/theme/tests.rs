//! Testes da paleta e do contraste.

use crate::syntax::STROKE_TOKENS;

use super::{
    MIN_STROKE_CONTRAST, MIN_TEXT_CONTRAST, PAPER, SLATE, Theme, Token, contrast_ratio,
    token_of_hex,
};

const FILLS: [Token; 6] = [
    Token::Paper,
    Token::Blue,
    Token::Green,
    Token::Amber,
    Token::Red,
    Token::Violet,
];

fn stroke_tokens() -> Vec<Token> {
    vec![
        Token::Paper,
        Token::Ink,
        Token::Muted,
        Token::Blue,
        Token::Green,
        Token::Amber,
        Token::Red,
        Token::Violet,
    ]
}

fn hex(theme: &Theme, token: Token) -> &'static str {
    theme.hex(token).expect("token com hex")
}

#[test]
fn text_meets_aa_on_every_fill() {
    for theme in [&PAPER, &SLATE] {
        for fill in FILLS {
            let ratio = contrast_ratio(theme.ink, hex(theme, fill));
            assert!(
                ratio >= MIN_TEXT_CONTRAST,
                "{}: ink sobre {} = {ratio}",
                theme.name,
                fill.name()
            );
        }
        assert!(
            contrast_ratio(theme.ink, theme.canvas) >= MIN_TEXT_CONTRAST,
            "{}: ink sobre canvas",
            theme.name
        );
    }
}

#[test]
fn ink_and_muted_strokes_meet_non_text_minimum() {
    for theme in [&PAPER, &SLATE] {
        for stroke in [Token::Ink, Token::Muted] {
            let against_canvas = contrast_ratio(hex(theme, stroke), theme.canvas);
            assert!(
                against_canvas >= MIN_STROKE_CONTRAST,
                "{}: {} sobre canvas = {against_canvas}",
                theme.name,
                stroke.name()
            );
            for fill in FILLS {
                let against_fill = contrast_ratio(hex(theme, stroke), hex(theme, fill));
                assert!(
                    against_fill >= MIN_STROKE_CONTRAST,
                    "{}: {} sobre {} = {against_fill}",
                    theme.name,
                    stroke.name(),
                    fill.name()
                );
            }
        }
    }
}

#[test]
fn colored_strokes_are_accents_below_minimum() {
    // Documenta o limite da paleta de §5.6: tokens coloridos não são contorno de estrutura.
    // Se a paleta mudar para valores conformes, este teste aponta a decisão a revisitar.
    for theme in [&PAPER, &SLATE] {
        for stroke in [
            Token::Blue,
            Token::Green,
            Token::Amber,
            Token::Red,
            Token::Violet,
        ] {
            let ratio = contrast_ratio(hex(theme, stroke), theme.canvas);
            assert!(
                ratio < MIN_STROKE_CONTRAST,
                "{}: {} sobre canvas = {ratio}",
                theme.name,
                stroke.name()
            );
        }
    }
}

#[test]
fn palette_covers_every_token_except_none() {
    for theme in [&PAPER, &SLATE] {
        for token in Token::ALL.into_iter().filter(|token| *token != Token::None) {
            assert!(
                theme.hex(token).is_some(),
                "{} sem {}",
                theme.name,
                token.name()
            );
            assert!(token.is_fill() || token.is_stroke(), "{}", token.name());
        }
    }
    assert!(PAPER.hex(Token::None).is_none());
    assert_eq!(PAPER.background(Token::None), PAPER.canvas);
}

#[test]
fn token_of_hex_roundtrips_across_themes() {
    for theme in [&PAPER, &SLATE] {
        for token in Token::ALL.into_iter().filter(|token| *token != Token::None) {
            let hex = theme.hex(token).expect("hex");
            assert_eq!(token_of_hex(hex), Some(token), "{hex}");
            assert_eq!(
                token_of_hex(&hex.to_uppercase()),
                Some(token),
                "{hex} maiúsculo"
            );
        }
    }
    assert_eq!(token_of_hex("#010203"), None);
}

#[test]
fn contrast_matches_known_extremes() {
    assert!((contrast_ratio("#000000", "#ffffff") - 21.0).abs() < 0.01);
    assert!((contrast_ratio("#ffffff", "#ffffff") - 1.0).abs() < 0.001);
    assert!((contrast_ratio("nao-e-hex", "#ffffff") - 1.0).abs() < 0.001);
}

#[test]
fn stroke_tokens_list_is_exhaustive() {
    let listed = stroke_tokens();
    let expected: Vec<Token> = STROKE_TOKENS
        .iter()
        .filter_map(|name| Token::from_name(name))
        .filter(|token| *token != Token::None)
        .collect();
    assert_eq!(listed, expected);
}
