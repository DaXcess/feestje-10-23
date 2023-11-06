mod blocks;
mod eyes;
mod falling;
mod sequence;
mod smile;
mod startup;
mod tictactoe;
mod time;

use embedded_graphics::{
    pixelcolor::{raw::ToBytes, Rgb888},
    prelude::{DrawTarget, OriginDimensions, Size},
};
use image::RgbImage;

pub trait Animation {
    fn should_execute(&self) -> bool;
    fn next_frame(&mut self) -> Option<RgbImage>;

    fn reload(&mut self) {}
}

#[derive(Clone)]
pub struct ImageBuffer(Vec<u8>);

impl ImageBuffer {
    pub fn new() -> Self {
        Self(vec![0; 3 * 64 * 32])
    }

    pub fn into_image(self) -> RgbImage {
        RgbImage::from_raw(64, 32, self.0).expect("Invalid buffer size")
    }
}

impl Default for ImageBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl OriginDimensions for ImageBuffer {
    fn size(&self) -> Size {
        Size {
            height: 32,
            width: 64,
        }
    }
}

impl DrawTarget for ImageBuffer {
    type Color = Rgb888;
    type Error = std::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> std::result::Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        pixels.into_iter().for_each(|p| {
            let index = p.0.y as usize * 64 + p.0.x as usize;
            let buff = &mut self.0[index * 3..index * 3 + 3];
            buff.copy_from_slice(&p.1.to_be_bytes());
        });

        Ok(())
    }
}

pub use blocks::*;
pub use eyes::*;
pub use falling::*;
pub use sequence::*;
pub use smile::*;
pub use startup::*;
pub use tictactoe::*;
pub use time::*;
