use std::time::Instant;

use image::{Rgb, RgbImage};
use rand::Rng;

use crate::image::hsl_to_rgb;

use super::Animation;

pub struct FallingAnimation {
    particles: Vec<(u8, u8, Rgb<u8>)>,

    last_frame: Instant,
}

impl FallingAnimation {
    pub fn new() -> Self {
        Self {
            particles: vec![],

            last_frame: Instant::now(),
        }
    }
}

impl Default for FallingAnimation {
    fn default() -> Self {
        Self::new()
    }
}

impl Animation for FallingAnimation {
    fn should_execute(&self) -> bool {
        // Target: ~60 FPS
        self.last_frame.elapsed().as_millis() > 50
    }

    fn next_frame(&mut self) -> Option<RgbImage> {
        // Move all particles one pixel down and remove particles that are off-screen
        self.particles.retain_mut(|(_, y, _)| {
            *y += 1;
            *y <= 42
        });

        // Add new particles randomly
        let mut rng = rand::thread_rng();

        // 2% chance every frame per X pixel
        for x in 0..64 {
            if rng.gen_range(0..100) < 2 {
                self.particles
                    .push((x, 0, hsl_to_rgb(rng.gen_range(0..360), 1.0, 0.5)));
            }
        }

        let mut image = RgbImage::new(64, 32);

        for (x, y, clr) in &self.particles {
            for i in (0..10).rev() {
                // Ignore out of screen trail
                if *y as i8 - i < 0 || *y as i8 - i >= 32 {
                    continue;
                }

                let modifier = (10.0 - i as f32) / 10.0;

                let (r, g, b) = (
                    (clr.0[0] as f32 * modifier) as u8,
                    (clr.0[1] as f32 * modifier) as u8,
                    (clr.0[2] as f32 * modifier) as u8,
                );

                image.put_pixel(*x as u32, *y as u32 - i as u32, Rgb([r, g, b]));
            }
        }

        self.last_frame = Instant::now();

        Some(image)
    }
}
