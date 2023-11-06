use std::time::Instant;

use image::{
    imageops::{resize, FilterType},
    Rgb, RgbImage,
};
use rand::Rng;

use crate::image::hsl_to_rgb;

use super::Animation;

// Arbritrarily increase size of panel, so that specific segment gaps can be achieved
const BUFFER_HEIGHT: u32 = 18;

pub struct BlocksAnimation {
    frame: u32,
    last_frame: Instant,

    segments: Vec<(u8, Rgb<u8>)>,
}

impl BlocksAnimation {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut segments = vec![];

        for i in 0..6 {
            let color = hsl_to_rgb(rng.gen_range(0..360), 1.0, 0.5);

            segments.push((i * 3, color));
        }

        Self {
            frame: 0,
            last_frame: Instant::now(),

            segments,
        }
    }
}

impl Default for BlocksAnimation {
    fn default() -> Self {
        Self::new()
    }
}

impl Animation for BlocksAnimation {
    fn should_execute(&self) -> bool {
        self.last_frame.elapsed().as_millis() > 20
    }

    fn next_frame(&mut self) -> Option<RgbImage> {
        let mut image = RgbImage::new(32, 16);

        if self.frame % 20 == 0 {
            self.segments.iter_mut().for_each(|segment| {
                let mut rng = rand::thread_rng();

                if segment.0 == 0 {
                    segment.0 = (BUFFER_HEIGHT - 1) as u8;
                    segment.1 = hsl_to_rgb(rng.gen_range(0..360), 1.0, 0.5);
                } else {
                    segment.0 -= 1;
                }
            });
        }

        let mul = (20.0 - self.frame as f32) / 20.0;
        let mul2 = self.frame as f32 / 20.0;

        for (pos, color) in &self.segments {
            let (color, color2) = (
                Rgb([
                    (color.0[0] as f32 * mul) as u8,
                    (color.0[1] as f32 * mul) as u8,
                    (color.0[2] as f32 * mul) as u8,
                ]),
                Rgb([
                    (color.0[0] as f32 * mul2) as u8,
                    (color.0[1] as f32 * mul2) as u8,
                    (color.0[2] as f32 * mul2) as u8,
                ]),
            );

            for x in 0..32 {
                if x % 5 < 2 {
                    set_pixel(&mut image, x, *pos as u32, color);
                } else {
                    set_pixel(
                        &mut image,
                        x,
                        (*pos as u32 - 1 + BUFFER_HEIGHT) % BUFFER_HEIGHT,
                        color,
                    );
                }

                if x % 5 < 2 {
                    set_pixel(
                        &mut image,
                        x,
                        (*pos as u32 - 1 + BUFFER_HEIGHT) % BUFFER_HEIGHT,
                        color2,
                    );
                } else {
                    set_pixel(
                        &mut image,
                        x,
                        (*pos as u32 - 2 + BUFFER_HEIGHT) % BUFFER_HEIGHT,
                        color2,
                    );
                }
            }
        }

        let resized = resize(&image, 64, 32, FilterType::Nearest);

        self.frame = (self.frame + 1) % 20;
        self.last_frame = Instant::now();

        Some(resized)
    }
}

fn set_pixel(image: &mut RgbImage, x: u32, y: u32, pixel: Rgb<u8>) {
    if x < image.width() && y < image.height() {
        image.put_pixel(x, y, pixel);
    }
}
