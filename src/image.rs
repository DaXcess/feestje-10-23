use std::io::Cursor;

use anyhow::{anyhow, Result};
use image::{
    codecs::{
        bmp::BmpDecoder,
        jpeg::{JpegDecoder, JpegEncoder},
    },
    imageops::FilterType,
};

// Re-export from crate with same name
pub use image::*;

const FRAME_DIM: usize = 72;
const FRAME_SIZE: usize = FRAME_DIM * FRAME_DIM;
const DECK_WIDTH: usize = 384;
// const DECK_HEIGHT: usize = 216;

pub enum ImageSourceType<'a> {
    Jpeg(&'a [u8]),
    JpegVec(Vec<u8>),
    RawRgb(&'a [u8]),
    Rgb(RgbImage),
}

impl<'a> ImageSourceType<'a> {
    pub fn to_raw(self) -> Result<Vec<u8>> {
        Ok(match self {
            ImageSourceType::Jpeg(jpeg) => decode_jpeg(jpeg)?,
            ImageSourceType::JpegVec(jpeg) => decode_jpeg(&jpeg)?,
            ImageSourceType::RawRgb(raw) => raw.to_vec(),
            ImageSourceType::Rgb(rgb) => rgb.as_raw().to_vec(),
        })
    }

    pub fn to_rgb(self, width: u32, height: u32) -> Result<RgbImage> {
        Ok(match self {
            ImageSourceType::Jpeg(_) | ImageSourceType::JpegVec(_) | ImageSourceType::RawRgb(_) => {
                ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, self.to_raw()?)
                    .ok_or(anyhow!("Buffer has wrong length"))?
            }
            ImageSourceType::Rgb(rgb) => rgb,
        })
    }
}

pub fn resize(image: &RgbImage, nwidth: u32, nheight: u32, hq: bool) -> RgbImage {
    image::imageops::resize(
        image,
        nwidth,
        nheight,
        if hq {
            FilterType::Gaussian
        } else {
            FilterType::Nearest
        },
    )
}

pub fn decode_bmp(image: &[u8]) -> Result<Vec<u8>> {
    let decoder = BmpDecoder::new(Cursor::new(image))?;
    let mut buffer = vec![0; decoder.total_bytes() as usize];
    decoder.read_image(&mut buffer)?;

    Ok(buffer)
}

pub fn decode_jpeg(image: &[u8]) -> Result<Vec<u8>> {
    let decoder = JpegDecoder::new(image)?;
    let mut buffer = vec![0; decoder.total_bytes() as usize];
    decoder.read_image(&mut buffer)?;

    Ok(buffer)
}

pub fn encode_jpeg(image: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let encoder = JpegEncoder::new(&mut buf);
    encoder.write_image(image, width, height, ColorType::Rgb8)?;

    Ok(buf)
}

pub fn split_raw_full_image(raw_image: &[u8]) -> Option<Vec<ImageBuffer<Rgb<u8>, Vec<u8>>>> {
    let mut buffers = vec![];

    for i in 0..15 {
        let mut frame = [0u8; FRAME_SIZE * 3];

        let row_idx = i / 5;
        let col_idx = i % 5;

        let start_row = row_idx * FRAME_DIM;
        let start_col = col_idx * FRAME_DIM;

        for j in 0..FRAME_DIM {
            let start_idx = (start_row + j) * DECK_WIDTH + start_col + 12;
            let end_idx = start_idx + FRAME_DIM;

            let target_start_idx = j * FRAME_DIM;
            let target_end_idx = (j + 1) * FRAME_DIM;

            frame[target_start_idx * 3..target_end_idx * 3]
                .copy_from_slice(&raw_image[start_idx * 3..end_idx * 3]);
        }

        let buffer = ImageBuffer::<Rgb<u8>, _>::from_raw(72, 72, frame[..].to_vec())?;

        buffers.push(buffer);
    }

    Some(buffers)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }

    p
}

pub fn hsl_to_rgb(h: u16, s: f32, l: f32) -> Rgb<u8> {
    let h = h as f32 / 360.0;
    let (r, g, b) = if s == 0.0 {
        (l, l, l)
    } else {
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        (
            hue_to_rgb(p, q, h + 1.0 / 3.0),
            hue_to_rgb(p, q, h),
            hue_to_rgb(p, q, h - 1.0 / 3.0),
        )
    };

    Rgb([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8])
}
