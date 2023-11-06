use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use streamdeck_hid_rs::ButtonState;

use crate::{image::ImageSourceType, matrix::animations::SmileAnimation, AppState};

const IMG_BACK: ImageSourceType = ImageSourceType::Jpeg(include_bytes!("../../images/back.jpg"));
const IMG_BACK_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/back_down.jpg"));

const IMG_CAMERA: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/camera.jpg"));
const IMG_CAMERA_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/camera_hold.jpg"));

const IMG_FLASH_PREVIEW: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/flash_preview.jpg"));
const IMG_FLASH_PREVIEW_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/flash_preview_down.jpg"));
const IMG_FLASH_ON: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/flash_on.jpg"));
const IMG_FLASH_ON_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/flash_on_down.jpg"));
const IMG_FLASH_OFF: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/flash_off.jpg"));
const IMG_FLASH_OFF_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/flash_off_down.jpg"));

const IMG_TIMER: ImageSourceType = ImageSourceType::Jpeg(include_bytes!("../../images/timer.jpg"));
const IMG_TIMER_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/timer_down.jpg"));
const IMG_TIMER_OFF: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/timer_off.jpg"));
const IMG_TIMER_OFF_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/timer_off_down.jpg"));

enum FlashState {
    Off,
    FlashOnly,
    FlashAndPreview,
}

impl FlashState {
    pub fn advance(&mut self) {
        match self {
            Self::Off => *self = Self::FlashAndPreview,
            Self::FlashOnly => *self = Self::Off,
            Self::FlashAndPreview => *self = Self::FlashOnly,
        }
    }

    pub fn image(&self, down: bool) -> ImageSourceType<'_> {
        match (self, down) {
            (Self::Off, true) => IMG_FLASH_OFF_DOWN,
            (Self::Off, false) => IMG_FLASH_OFF,
            (Self::FlashOnly, true) => IMG_FLASH_ON_DOWN,
            (Self::FlashOnly, false) => IMG_FLASH_ON,
            (Self::FlashAndPreview, true) => IMG_FLASH_PREVIEW_DOWN,
            (Self::FlashAndPreview, false) => IMG_FLASH_PREVIEW,
        }
    }

    pub fn is_flash(&self) -> bool {
        match self {
            Self::FlashAndPreview | Self::FlashOnly => true,
            Self::Off => false,
        }
    }

    pub fn is_preview(&self) -> bool {
        match self {
            Self::FlashAndPreview => true,
            Self::FlashOnly | Self::Off => false,
        }
    }
}

pub fn launch(state: &mut AppState) -> Result<()> {
    let mut flash = FlashState::FlashAndPreview;
    let mut timer = false;

    state.deck.clear()?;

    state.deck.set_button_image(0, IMG_BACK)?;
    state.deck.set_button_image(2, IMG_TIMER_OFF)?;
    state.deck.set_button_image(3, flash.image(false))?;
    state.deck.set_button_image(4, IMG_CAMERA)?;

    loop {
        let message = state.deck.next_btn_event()?;

        match message.button_id {
            0 => {
                if matches!(message.state, ButtonState::Up) {
                    break;
                } else {
                    state
                        .deck
                        .set_button_image(message.button_id as u8, IMG_BACK_DOWN)?;
                }
            }

            2 => {
                if matches!(message.state, ButtonState::Up) {
                    timer = !timer;

                    state.deck.set_button_image(
                        message.button_id as u8,
                        if timer { IMG_TIMER } else { IMG_TIMER_OFF },
                    )?;
                } else {
                    state.deck.set_button_image(
                        message.button_id as u8,
                        if timer {
                            IMG_TIMER_DOWN
                        } else {
                            IMG_TIMER_OFF_DOWN
                        },
                    )?;
                }
            }

            3 => {
                if matches!(message.state, ButtonState::Up) {
                    flash.advance();

                    state
                        .deck
                        .set_button_image(message.button_id as u8, flash.image(false))?;
                } else {
                    state
                        .deck
                        .set_button_image(message.button_id as u8, flash.image(true))?;
                }
            }

            4 => {
                if matches!(message.state, ButtonState::Up) {
                    state.deck.clear()?;

                    // Capture image, store on disk and display on StreamDeck
                    capture_image(state, flash.is_flash(), flash.is_preview(), timer)?;
                    state.deck.flush_btn_events()?;

                    // Press any button to clear image and go back to camera interface
                    state.deck.wait_for_any_press()?;
                    state.deck.clear()?;

                    if flash.is_preview() || timer {
                        state.matrix.clear()?;
                        state.start_matrix_animation();
                    }

                    state.deck.set_button_image(0, IMG_BACK)?;
                    state
                        .deck
                        .set_button_image(2, if timer { IMG_TIMER } else { IMG_TIMER_OFF })?;
                    state.deck.set_button_image(3, flash.image(false))?;
                    state.deck.set_button_image(4, IMG_CAMERA)?;
                } else {
                    state
                        .deck
                        .set_button_image(message.button_id as u8, IMG_CAMERA_DOWN)?;
                }
            }

            _ => {}
        }
    }

    Ok(())
}

fn capture_image(state: &mut AppState, flash: bool, preview: bool, timer: bool) -> Result<()> {
    if timer {
        state
            .matrix
            .set_animation(Box::new(SmileAnimation::new()))?;
        std::thread::sleep(Duration::from_secs(4));
    }

    let image = crate::camera::capture_image(state, flash)?;

    if !preview && flash && !timer {
        state.start_matrix_animation();
    }

    let start = SystemTime::now();
    let unix = start.duration_since(UNIX_EPOCH)?.as_secs();

    let filename = format!("captures/{unix}.jpg");
    let jpeg = crate::image::encode_jpeg(&image, image.width(), image.height())?;
    std::fs::write(&filename, jpeg)?;

    std::thread::spawn({
        let filename = filename.clone();

        move || crate::cloud::upload_file(&filename).ok()
    });

    let resized = crate::image::resize(&image, 384, 216, true);
    state
        .deck
        .set_fullscreen_image(ImageSourceType::Rgb(resized))?;

    if preview {
        let resized = crate::image::resize(&image, 64, 32, true);
        state.matrix.set_image(resized)?;
    }

    Ok(())
}
