use std::time::Instant;

use image::RgbImage;
use rand::Rng;

use super::Animation;

const BITMAP_SIZE: usize = 64 * 32 * 3;
const EYES_IMAGE_DATA: &[u8] = include_bytes!("../../../images/eyes.raw");

// 0: eyes_closed
// 1: eyes_default
// 2: eyes_look_apart
// 3: eyes_look_crossed
// 4: eyes_look_left
// 5: eyes_look_right

pub struct EyesAnimation {
    frame: u32,
    eyes_closed_frames: u32,
    next_eyes_frame: u32,

    last_frame: Instant,

    current_eye: u8,
}

impl EyesAnimation {
    pub fn new() -> Self {
        Self {
            frame: 0,
            eyes_closed_frames: 0,
            next_eyes_frame: rand::thread_rng().gen_range(30..75),

            last_frame: Instant::now(),

            current_eye: 1,
        }
    }
}

impl Default for EyesAnimation {
    fn default() -> Self {
        Self::new()
    }
}

impl Animation for EyesAnimation {
    fn should_execute(&self) -> bool {
        // Target 10 FPS
        self.last_frame.elapsed().as_millis() > 100
    }

    fn next_frame(&mut self) -> Option<image::RgbImage> {
        let mut rng = rand::thread_rng();

        if self.frame > self.next_eyes_frame {
            self.frame = 0;
            self.eyes_closed_frames = 1;
            self.current_eye = 0;
            self.next_eyes_frame = rng.gen_range(30..75);
        }

        if self.eyes_closed_frames > 0 {
            self.eyes_closed_frames -= 1;
            self.last_frame = Instant::now();

            return Some(get_image_from_index(0));
        }

        if self.current_eye == 0 {
            // Pick random eye

            let number = rng.gen_range(0..=100u32);
            self.current_eye = match number {
                0..=30 => 1,
                31..=60 => 4,
                61..=90 => 5,
                91..=95 => 2,
                96.. => 3,
            }
        }

        self.frame += 1;
        self.last_frame = Instant::now();

        Some(get_image_from_index(self.current_eye as usize))
    }
}

fn get_image_from_index(idx: usize) -> RgbImage {
    if idx > 5 {
        panic!("Index out of range 0-5");
    }

    RgbImage::from_raw(
        64,
        32,
        EYES_IMAGE_DATA[BITMAP_SIZE * idx..BITMAP_SIZE * (idx + 1)].to_vec(),
    )
    .expect("Buffer has invalid length")
}
