pub mod animations;
pub mod color_utils;
pub mod iter;

use std::{
    ops::{Deref, DerefMut},
    sync::mpsc::{Receiver, SyncSender, TryRecvError},
};

use anyhow::{anyhow, Result};
use embedded_graphics::{
    pixelcolor::{raw::ToBytes, Rgb888},
    prelude::{DrawTarget, OriginDimensions, RgbColor, Size},
};
use image::RgbImage;
use rpi_led_panel::{Canvas, HardwareMapping, RGBMatrix, RGBMatrixConfig};

use animations::Animation;

#[cfg(debug_assertions)]
const SLOWDOWN: u32 = 2;

#[cfg(not(debug_assertions))]
const SLOWDOWN: u32 = 3;

const MATRIX_WIDTH: usize = 64;
const MATRIX_ROWS: usize = 32;
const RGB_BYTE_LENGTH: usize = 3;

const MATRIX_FULL_IMAGE_LENGTH: usize = MATRIX_WIDTH * MATRIX_ROWS * RGB_BYTE_LENGTH;

const DEFAULT_BRIGHTNESS: u8 = 100;

enum State {
    Noop,
    Solid(u8, u8, u8),
    Image(RgbImage),
    Animation(Box<dyn Animation + Send + Sync>),
}

enum SchedulerCommand {
    UpdateState(State),
    SetBrightness(u8),
}

pub struct Matrix {
    tx: SyncSender<SchedulerCommand>,
    rgb_buffer: [u8; MATRIX_FULL_IMAGE_LENGTH],
}

impl Clone for Matrix {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rgb_buffer: [0; MATRIX_FULL_IMAGE_LENGTH],
        }
    }
}

impl Deref for Matrix {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.rgb_buffer
    }
}

impl DerefMut for Matrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rgb_buffer
    }
}

impl Matrix {
    pub fn open() -> Result<Self> {
        let config = RGBMatrixConfig {
            cols: MATRIX_WIDTH,
            rows: MATRIX_ROWS,
            hardware_mapping: HardwareMapping::regular(),
            slowdown: Some(SLOWDOWN),
            ..Default::default()
        };

        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let (matrix, canvas) = RGBMatrix::new(config, 0)?;

        std::thread::spawn(move || scheduler(matrix, canvas, rx));

        Ok(Self {
            tx,
            rgb_buffer: [0; MATRIX_FULL_IMAGE_LENGTH],
        })
    }

    /// Takes the current [`Self::rgb_buffer`] and writes it as an image to the scheduler
    pub fn flush_image(&self) -> Result<()> {
        let image = RgbImage::from_raw(64, 32, self.rgb_buffer.to_vec()).expect("huh");

        self.tx
            .send(SchedulerCommand::UpdateState(State::Image(image)))?;

        Ok(())
    }

    pub fn set_image(&self, image: RgbImage) -> Result<()> {
        if image.len() != MATRIX_FULL_IMAGE_LENGTH {
            return Err(anyhow!(
                "Invalid image size, image must be exactly {MATRIX_FULL_IMAGE_LENGTH} RGB bytes",
            ));
        }

        self.tx
            .send(SchedulerCommand::UpdateState(State::Image(image)))?;

        Ok(())
    }

    pub fn set_animation(&self, animation: Box<dyn Animation + Send + Sync>) -> Result<()> {
        self.tx
            .send(SchedulerCommand::UpdateState(State::Animation(animation)))?;
        Ok(())
    }

    pub fn fill(&self, r: u8, g: u8, b: u8) -> Result<()> {
        self.tx
            .send(SchedulerCommand::UpdateState(State::Solid(r, g, b)))?;
        Ok(())
    }

    pub fn set_brightness(&mut self, brightness: u8) -> Result<()> {
        let brightness = std::cmp::min(brightness, 100);

        self.tx.send(SchedulerCommand::SetBrightness(brightness))?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        self.tx
            .send(SchedulerCommand::UpdateState(State::Solid(0, 0, 0)))?;
        Ok(())
    }
}

impl OriginDimensions for Matrix {
    fn size(&self) -> Size {
        Size {
            height: 32,
            width: 64,
        }
    }
}

impl DrawTarget for Matrix {
    type Color = Rgb888;
    type Error = std::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> std::result::Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        pixels.into_iter().for_each(|p| {
            let index = p.0.y as usize * MATRIX_WIDTH + p.0.x as usize;
            let buff = &mut self.rgb_buffer[index * 3..index * 3 + 3];
            buff.copy_from_slice(&p.1.to_be_bytes());
        });

        Ok(())
    }
}

fn scheduler(mut matrix: RGBMatrix, mut canvas: Box<Canvas>, rx: Receiver<SchedulerCommand>) {
    let mut brightness = DEFAULT_BRIGHTNESS;
    let mut state: State = State::Noop;

    loop {
        match rx.try_recv() {
            Ok(command) => match command {
                SchedulerCommand::SetBrightness(new_brightness) => brightness = new_brightness,
                SchedulerCommand::UpdateState(new_state) => state = new_state,
            },
            Err(TryRecvError::Disconnected) => break,
            Err(TryRecvError::Empty) => {}
        }

        canvas.set_brightness(brightness);
        update_state_canvas(&mut canvas, &mut state);
        matrix.update_on_vsync(canvas.clone());
    }
}

fn update_state_canvas(canvas: &mut Box<Canvas>, state: &mut State) {
    match state {
        State::Noop => {}
        State::Solid(r, g, b) => {
            canvas.fill(*r, *g, *b);
        }
        State::Image(image) => {
            if image.len() != MATRIX_FULL_IMAGE_LENGTH {
                *state = State::Noop;
                return;
            }

            for y in 0..MATRIX_ROWS {
                for x in 0..MATRIX_WIDTH {
                    let pixel = image.get_pixel(x as u32, y as u32);

                    canvas.set_pixel(x, y, pixel.0[0], pixel.0[1], pixel.0[2]);
                }
            }
        }
        State::Animation(animation) => {
            if !animation.should_execute() {
                return;
            }

            let Some(image) = animation.next_frame() else {
                // When animation has finished and looping is disabled
                *state = State::Noop;
                return;
            };

            if image.len() != MATRIX_FULL_IMAGE_LENGTH {
                *state = State::Noop;
                return;
            }

            canvas.clear(Rgb888::BLACK).unwrap();

            for y in 0..MATRIX_ROWS {
                for x in 0..MATRIX_WIDTH {
                    let pixel = image.get_pixel(x as u32, y as u32);

                    canvas.set_pixel(x, y, pixel.0[0], pixel.0[1], pixel.0[2]);
                }
            }
        }
    }
}
