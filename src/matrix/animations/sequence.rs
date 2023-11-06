use std::time::{Duration, Instant};

use super::{Animation, BlocksAnimation, EyesAnimation, FallingAnimation, TimeAnimation};

pub struct SequenceAnimation {
    time_animation: TimeAnimation,
    eyes_animation: EyesAnimation,
    falling_animation: FallingAnimation,
    blocks_animation: BlocksAnimation,

    current_animation: u8,
    animation_start: Instant,
}

impl SequenceAnimation {
    pub fn new() -> Self {
        Self {
            time_animation: TimeAnimation::new(false),
            eyes_animation: EyesAnimation::new(),
            falling_animation: FallingAnimation::new(),
            blocks_animation: BlocksAnimation::new(),

            current_animation: 0,
            animation_start: Instant::now(),
        }
    }
}

impl Default for SequenceAnimation {
    fn default() -> Self {
        Self::new()
    }
}

impl Animation for SequenceAnimation {
    fn should_execute(&self) -> bool {
        match self.current_animation {
            0 => self.time_animation.should_execute(),
            1 => self.eyes_animation.should_execute(),
            2 => self.falling_animation.should_execute(),
            3 => self.blocks_animation.should_execute(),
            _ => false,
        }
    }

    fn next_frame(&mut self) -> Option<image::RgbImage> {
        // Will not be exactly 30 seconds, as `should_execute` will cause time drift
        if self.animation_start.elapsed() > Duration::from_secs(30) {
            self.current_animation = (self.current_animation + 1) % 4;
            self.animation_start = Instant::now();

            match self.current_animation {
                0 => self.time_animation.reload(),
                1 => self.eyes_animation.reload(),
                2 => self.falling_animation.reload(),
                3 => self.blocks_animation.reload(),
                _ => {}
            }
        }

        match self.current_animation {
            0 => self.time_animation.next_frame(),
            1 => self.eyes_animation.next_frame(),
            2 => self.falling_animation.next_frame(),
            3 => self.blocks_animation.next_frame(),
            _ => None,
        }
    }
}
