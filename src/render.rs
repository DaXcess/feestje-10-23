use std::io::Cursor;

use anyhow::Result;
use image::{GenericImage, ImageFormat, Rgb, RgbImage, RgbaImage};
use text_to_png::{FontSize, TextRenderer};

const FONT_DATA: &[u8] = include_bytes!("../assets/font.ttf");

pub fn render_text<T: AsRef<str>, S: TryInto<FontSize>>(text: T, size: S) -> Result<RgbImage> {
    let renderer = TextRenderer::try_new_with_ttf_font_data(FONT_DATA)?;
    let text_png = renderer.render_text_to_png_data(text, size, "#ffffff")?;

    let image = image::load(Cursor::new(&text_png.data), ImageFormat::Png)?;

    let rgba = image.to_rgba8();

    let mut target = RgbaImage::new(72, 72);
    let x = (72 - text_png.size.width) / 2;
    let y = (72 - text_png.size.height) / 2;

    target.copy_from(&rgba, x, y)?;

    let mut rgb_img = RgbImage::new(72, 72);
    for y in 0..72 {
        for x in 0..72 {
            let pixel = target.get_pixel(x, y);

            let alpha = pixel[3] as f32 / 255.0;
            let blended_color = Rgb([
                (pixel[0] as f32 * alpha) as u8,
                (pixel[1] as f32 * alpha) as u8,
                (pixel[2] as f32 * alpha) as u8,
            ]);

            rgb_img.put_pixel(x, y, blended_color);
        }
    }

    Ok(rgb_img)
}
