pub mod tictactoe;

use crate::{image::ImageSourceType, AppState};
use anyhow::Result;
use streamdeck_hid_rs::ButtonState;

const IMG_BACK: ImageSourceType = ImageSourceType::Jpeg(include_bytes!("../../images/back.jpg"));
const IMG_BACK_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/back_down.jpg"));

const IMG_TICTACTOE: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/tictactoe.jpg"));
const IMG_TICTACTOE_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/tictactoe_down.jpg"));

pub fn launch(state: &mut AppState) -> Result<()> {
    state.deck.clear()?;

    state.deck.set_button_image(0, IMG_BACK)?;
    state.deck.set_button_image(4, IMG_TICTACTOE)?;

    loop {
        let message = state.deck.next_btn_event()?;
        let id = message.button_id as u8;

        match id {
            0 => {
                if matches!(message.state, ButtonState::Up) {
                    break;
                } else {
                    state
                        .deck
                        .set_button_image(message.button_id as u8, IMG_BACK_DOWN)?;
                }
            }

            4 => {
                if matches!(message.state, ButtonState::Up) {
                    tictactoe::launch(state)?;

                    state.start_matrix_animation();

                    state.deck.clear()?;
                    state.deck.flush_btn_events()?;
                    state.deck.set_button_image(0, IMG_BACK)?;
                    state.deck.set_button_image(4, IMG_TICTACTOE)?;
                } else {
                    state.deck.set_button_image(4, IMG_TICTACTOE_DOWN)?;
                }
            }

            _ => {}
        }
    }

    Ok(())
}
