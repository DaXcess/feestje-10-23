use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use image::RgbImage;

use crate::image;

use super::Animation;

const IMG_FERRIS: &[u8] = include_bytes!("../../../images/ferris.bmp");

pub struct StartupAnimation {
    image: RgbImage,

    last_frame: Instant,
    frame: u32,
}

impl StartupAnimation {
    pub fn load() -> Result<Self> {
        let image = RgbImage::from_raw(30, 21, image::decode_bmp(IMG_FERRIS)?)
            .ok_or(anyhow!("Buffer has wrong length"))?;

        Ok(Self {
            image,

            last_frame: Instant::now(),
            frame: 0,
        })
    }
}

impl Animation for StartupAnimation {
    fn should_execute(&self) -> bool {
        self.frame <= 30 && self.last_frame.elapsed() > Duration::from_millis(25)
    }

    fn next_frame(&mut self) -> Option<RgbImage> {
        let mut result = RgbImage::new(64, 32);

        match self.frame {
            0..=30 => {
                let width = self.frame;

                for x in 0..width {
                    for y in 0..21 {
                        let pixel = self.image.get_pixel(x, y);
                        result.put_pixel(x + 17, y + 5, *pixel);
                    }
                }
            }

            _ => {
                for x in 0..30 {
                    for y in 0..21 {
                        let pixel = self.image.get_pixel(x, y);
                        result.put_pixel(x + 17, y + 5, *pixel);
                    }
                }
            }
        }

        self.last_frame = Instant::now();
        self.frame += 1;

        Some(result)
    }
}
