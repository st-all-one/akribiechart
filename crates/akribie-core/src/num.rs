//! Formatação numérica canônica.
//!
//! Todo número emitido pelo binário (`.akrc` canônico, SVG, `scene` JSON, `.excalidraw`) passa por
//! aqui. `f32::to_string` não serve: produz `0.30000001192092896` e depende da versão da std. A
//! regra é fixa — até duas casas, zeros à direita removidos — e é contrato de bytes (ADR-0001 §2).

/// Formata um número com até duas casas decimais, sem zeros à direita.
pub fn format(value: f32) -> String {
    if !value.is_finite() {
        // Não deveria acontecer: o parser rejeita não-finitos. Preferimos um valor estável a "NaN".
        return "0".to_string();
    }
    let rounded = (value * 100.0).round() / 100.0;
    let mut text = format!("{rounded:.2}");
    while text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text == "-0" {
        text = "0".to_string();
    }
    text
}

/// Número JSON a partir do texto canônico: `214`, não `214.0`.
pub fn json(value: f32) -> serde_json::Value {
    format(value)
        .parse::<serde_json::Number>()
        .map_or_else(|_| serde_json::Value::from(0), serde_json::Value::Number)
}

#[cfg(test)]
mod tests {
    use super::format;

    #[test]
    fn removes_trailing_zeros() {
        assert_eq!(format(120.0), "120");
        assert_eq!(format(120.5), "120.5");
        assert_eq!(format(0.25), "0.25");
        assert_eq!(format(-0.0), "0");
        assert_eq!(format(2.999), "3");
    }
}
