pub mod camera;
pub mod cloud;
pub mod deck;
pub mod emoji;
pub mod image;
pub mod matrix;
pub mod menus;
pub mod render;
pub mod state;

use std::time::{Duration, Instant};

use crate::deck::Deck;
use anyhow::Result;
use image::ImageSourceType;
use libcamera::logging::{log_set_target, LoggingTarget};
use matrix::{animations::StartupAnimation, Matrix};
use state::{AppState, MatrixAnimation};

const SD_ERROR_IMAGE: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../images/error.jpg"));

fn main() -> Result<()> {
    log_set_target(LoggingTarget::None).ok();

    let matrix = Matrix::open()?;
    let deck = Deck::open()?;
    let animation = StartupAnimation::load()?;

    deck.start_event_loop();
    matrix.set_animation(Box::new(animation))?;

    let now = Instant::now();

    // Perform loading tasks here
    let emojis = emoji::load_emojis()?;

    // Make sure animation takes at least 3 secs
    std::thread::sleep(
        Duration::from_secs(3)
            .checked_sub(now.elapsed())
            .unwrap_or(Duration::from_millis(1)),
    );

    // Loading complete

    let mut state = AppState {
        deck,
        matrix,
        emojis,

        matrix_animation: MatrixAnimation::Sequence,
    };

    state.start_matrix_animation();
    state.deck.flush_btn_events()?;

    if let Err(why) = menus::main::launch(&mut state) {
        // Broke out of main menu, something went wrong?
        state.deck.set_fullscreen_image(SD_ERROR_IMAGE).ok();

        panic!("Unexpected exit from main menu: {}", why.backtrace());
    }

    Ok(())
}
