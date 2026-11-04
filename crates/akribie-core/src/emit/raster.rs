//! Rasterização determinística: SVG → PNG/JPEG/AVIF.
//!
//! As fontes são **injetadas** no `usvg` a partir dos mesmos bytes embutidos que medem o layout. Sem
//! isso, o raster usaria a fonte do sistema e o texto sairia com outras métricas. AVIF é codificado
//! com uma thread fixa — paralelismo variável muda os bytes.

use image::codecs::{avif::AvifEncoder, jpeg::JpegEncoder};
use image::{ExtendedColorType, ImageEncoder};

use crate::error::{Error, Result};
use crate::text::{FONT_BYTES, SANS_BYTES};

/// Formato raster suportado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterFormat {
    /// PNG com canal alfa.
    Png,
    /// JPEG opaco.
    Jpeg,
    /// AVIF opaco.
    Avif,
}

impl RasterFormat {
    /// Formato pelo nome (`png`, `jpg`/`jpeg`, `avif`).
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "avif" => Some(Self::Avif),
            _ => None,
        }
    }

    /// Extensão canônica do arquivo.
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Avif => "avif",
        }
    }

    /// `true` quando o formato guarda transparência.
    pub const fn keeps_alpha(self) -> bool {
        matches!(self, Self::Png)
    }
}

/// Rasteriza o SVG no formato pedido.
///
/// # Errors
/// `Internal` quando o SVG não parseia ou o codificador falha (SVG gerado pelo próprio akribie é
/// sempre válido; falhar aqui é bug de invariante).
pub fn encode(svg: &str, format: RasterFormat, scale: f32) -> Result<Vec<u8>> {
    let mut options = usvg::Options::default();
    options.fontdb_mut().load_font_data(FONT_BYTES.to_vec());
    options.fontdb_mut().load_font_data(SANS_BYTES.to_vec());
    options.font_family = "Liberation Sans".to_string();
    let tree = usvg::Tree::from_str(svg, &options)
        .map_err(|error| Error::internal(format!("SVG gerado é inválido: {error}")))?;
    let size = tree.size();
    let scale = scale.max(0.1);
    let width = pixels(size.width() * scale)?;
    let height = pixels(size.height() * scale)?;
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| Error::internal(format!("canvas {width}x{height} grande demais")))?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    match format {
        RasterFormat::Png => pixmap
            .encode_png()
            .map_err(|error| Error::internal(format!("falha ao codificar PNG: {error}"))),
        RasterFormat::Jpeg => {
            let data = rgb(&pixmap);
            let mut buffer = Vec::new();
            JpegEncoder::new_with_quality(&mut buffer, 90)
                .write_image(&data, width, height, ExtendedColorType::Rgb8)
                .map_err(|error| Error::internal(format!("falha ao codificar JPEG: {error}")))?;
            Ok(buffer)
        }
        RasterFormat::Avif => {
            let data = rgb(&pixmap);
            let mut buffer = Vec::new();
            AvifEncoder::new_with_speed_quality(&mut buffer, 6, 80)
                .with_num_threads(Some(1))
                .write_image(&data, width, height, ExtendedColorType::Rgb8)
                .map_err(|error| Error::internal(format!("falha ao codificar AVIF: {error}")))?;
            Ok(buffer)
        }
    }
}

/// Canais RGB a partir do pixmap RGBA (o canvas é opaco; o alfa é sempre 255).
fn rgb(pixmap: &tiny_skia::Pixmap) -> Vec<u8> {
    let data = pixmap.data();
    let mut rgb = Vec::with_capacity(data.len().checked_div(4).unwrap_or(0).saturating_mul(3));
    let (pixels, _) = data.as_chunks::<4>();
    for [red, green, blue, _] in pixels {
        rgb.extend_from_slice(&[*red, *green, *blue]);
    }
    rgb
}

/// Dimensão em pixels: inteiro positivo, arredondado para cima.
fn pixels(value: f32) -> Result<u32> {
    let rounded = value.ceil().max(1.0);
    if rounded > 16_384.0 {
        return Err(Error::internal(format!(
            "dimensão {rounded} excede o limite de 16384 px"
        )));
    }
    Ok(to_u32(rounded))
}

#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "interop de raster: a dimensão já foi validada em [1, 16384]"
)]
fn to_u32(value: f32) -> u32 {
    value as u32
}

#[cfg(test)]
mod tests {
    use super::{RasterFormat, encode};

    const TINY: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"20\" height=\"10\" viewBox=\"0 0 20 10\"><rect width=\"20\" height=\"10\" fill=\"#ffffff\"/><rect x=\"2\" y=\"2\" width=\"6\" height=\"6\" fill=\"#1e1e1e\"/></svg>";

    #[test]
    fn png_scales_with_the_factor() {
        let half = encode(TINY, RasterFormat::Png, 1.0).expect("png");
        let double = encode(TINY, RasterFormat::Png, 2.0).expect("png");
        assert_eq!(
            &half[..8],
            &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]
        );
        // Cabeçalho IHDR: largura e altura em big-endian nos offsets 16..24.
        let width = u32::from_be_bytes([half[16], half[17], half[18], half[19]]);
        let doubled = u32::from_be_bytes([double[16], double[17], double[18], double[19]]);
        assert_eq!(width, 20);
        assert_eq!(doubled, 40);
    }

    #[test]
    fn encoding_is_deterministic() {
        for format in [RasterFormat::Png, RasterFormat::Jpeg, RasterFormat::Avif] {
            let first = encode(TINY, format, 1.5).expect("primeira");
            let second = encode(TINY, format, 1.5).expect("segunda");
            assert_eq!(first, second, "{format:?}");
        }
    }

    #[test]
    fn invalid_svg_is_an_internal_error() {
        let error = encode("<svg", RasterFormat::Png, 1.0).expect_err("deve falhar");
        assert_eq!(error.exit_code(), 70);
    }
}
