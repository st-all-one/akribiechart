//! Cálculo de contraste WCAG 2.2.

/// Luminância relativa de um hex `#rrggbb` (WCAG 2.2).
///
/// Hex inválido devolve `None`, o que faz [`contrast_ratio`] devolver o mínimo (1.0): um erro de
/// paleta nunca vira "contraste alto" por acidente.
fn luminance(hex: &str) -> Option<f32> {
    let mut channels = [0.0_f32; 3];
    for (index, channel) in channels.iter_mut().enumerate() {
        let start = index.saturating_mul(2).saturating_add(1);
        let pair = hex.get(start..start.saturating_add(2))?;
        let value = u8::from_str_radix(pair, 16).ok()?;
        let normalized = f32::from(value) / 255.0;
        *channel = if normalized <= 0.040_45 {
            normalized / 12.92
        } else {
            ((normalized + 0.055) / 1.055).powf(2.4)
        };
    }
    let [red, green, blue] = channels;
    Some(0.2126 * red + 0.7152 * green + 0.0722 * blue)
}

/// Razão de contraste WCAG entre dois hex `#rrggbb` (1.0 … 21.0).
pub fn contrast_ratio(first: &str, second: &str) -> f32 {
    let (Some(a), Some(b)) = (luminance(first), luminance(second)) else {
        return 1.0;
    };
    let (high, low) = if a >= b { (a, b) } else { (b, a) };
    (high + 0.05) / (low + 0.05)
}
