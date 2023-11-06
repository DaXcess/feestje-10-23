use std::time::Instant;

use image::RgbImage;

use crate::image::decode_bmp;

use super::Animation;

const SMILE_IMAGE: &[u8] = include_bytes!("../../../images/smile.bmp");

pub struct SmileAnimation {
    image: RgbImage,
    last_frame: Instant,

    offset: isize,
}

impl SmileAnimation {
    pub fn new() -> Self {
        let image = RgbImage::from_raw(
            64 * 4,
            32,
            decode_bmp(SMILE_IMAGE).expect("Smile image corrupt"),
        )
        .expect("Smile image corrupt");

        Self {
            image,
            last_frame: Instant::now(),

            offset: 44,
        }
    }
}

impl Default for SmileAnimation {
    fn default() -> Self {
        Self::new()
    }
}

impl Animation for SmileAnimation {
    fn should_execute(&self) -> bool {
        // Target: ~64FPS
        self.last_frame.elapsed().as_millis() > 15 && self.offset >= -192
    }

    fn next_frame(&mut self) -> Option<image::RgbImage> {
        let mut image = RgbImage::new(64, 32);

        let x_dst = std::cmp::max(0, self.offset);
        let x_src = std::cmp::max(0, -self.offset);

        for y in 0..32 {
            for x in 0..64 - x_dst {
                let pixel = self.image.get_pixel((x_src + x) as u32, y);
                image.put_pixel((x_dst + x) as u32, y, *pixel);
            }
        }

        self.offset -= 1;
        self.last_frame = Instant::now();

        Some(image)
    }
}
