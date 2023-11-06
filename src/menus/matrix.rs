mod double_emoji;
mod emoji;

use crate::{image::ImageSourceType, state::MatrixAnimation, AppState};
use anyhow::Result;
use streamdeck_hid_rs::ButtonState;

const IMG_BACK: ImageSourceType = ImageSourceType::Jpeg(include_bytes!("../../images/back.jpg"));
const IMG_BACK_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/back_down.jpg"));

const IMG_EMOJI: ImageSourceType = ImageSourceType::Jpeg(include_bytes!("../../images/emoji.jpg"));
const IMG_EMOJI_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/emoji_down.jpg"));

const IMG_DOUBLE_EMOJI: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/double_emoji.jpg"));
const IMG_DOUBLE_EMOJI_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/double_emoji_down.jpg"));

// All Animations

const IMG_CLOCK: ImageSourceType = ImageSourceType::Jpeg(include_bytes!("../../images/clock.jpg"));
const IMG_CLOCK_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/clock_down.jpg"));

const IMG_EYES: ImageSourceType = ImageSourceType::Jpeg(include_bytes!("../../images/eyes.jpg"));
const IMG_EYES_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/eyes_down.jpg"));

const IMG_FALLING: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/falling.jpg"));
const IMG_FALLING_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/falling_down.jpg"));

const IMG_BLOCKS: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/blocks.jpg"));
const IMG_BLOCKS_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/blocks_down.jpg"));

const IMG_SEQUENCE: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/sequence.jpg"));
const IMG_SEQUENCE_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/sequence_down.jpg"));

pub fn launch(state: &mut AppState) -> Result<()> {
    state.deck.clear()?;

    state.deck.set_button_image(0, IMG_BACK)?;
    state.deck.set_button_image(3, IMG_DOUBLE_EMOJI)?;
    state.deck.set_button_image(4, IMG_EMOJI)?;

    render_animation_items(state)?;

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

            3 => {
                if matches!(message.state, ButtonState::Up) {
                    state.matrix.clear()?;

                    double_emoji::launch(state)?;

                    state.start_matrix_animation();

                    state.deck.clear()?;
                    state.deck.flush_btn_events()?;
                    state.deck.set_button_image(0, IMG_BACK)?;
                    state.deck.set_button_image(3, IMG_DOUBLE_EMOJI)?;
                    state.deck.set_button_image(4, IMG_EMOJI)?;

                    render_animation_items(state)?;
                } else {
                    state.deck.set_button_image(3, IMG_DOUBLE_EMOJI_DOWN)?;
                }
            }

            4 => {
                if matches!(message.state, ButtonState::Up) {
                    state.matrix.clear()?;

                    emoji::launch(state)?;

                    state.start_matrix_animation();

                    state.deck.clear()?;
                    state.deck.flush_btn_events()?;
                    state.deck.set_button_image(0, IMG_BACK)?;
                    state.deck.set_button_image(3, IMG_DOUBLE_EMOJI)?;
                    state.deck.set_button_image(4, IMG_EMOJI)?;

                    render_animation_items(state)?;
                } else {
                    state.deck.set_button_image(4, IMG_EMOJI_DOWN)?;
                }
            }

            10 => {
                if matches!(message.state, ButtonState::Up) {
                    state.deck.set_button_image(10, IMG_SEQUENCE)?;

                    state.matrix_animation = MatrixAnimation::Sequence;
                    state.start_matrix_animation();
                } else {
                    state.deck.set_button_image(10, IMG_SEQUENCE_DOWN)?;
                }
            }

            11 => {
                if matches!(message.state, ButtonState::Up) {
                    state.deck.set_button_image(11, IMG_BLOCKS)?;

                    state.matrix_animation = MatrixAnimation::Blocks;
                    state.start_matrix_animation();
                } else {
                    state.deck.set_button_image(11, IMG_BLOCKS_DOWN)?;
                }
            }

            12 => {
                if matches!(message.state, ButtonState::Up) {
                    state.deck.set_button_image(12, IMG_FALLING)?;

                    state.matrix_animation = MatrixAnimation::Falling;
                    state.start_matrix_animation();
                } else {
                    state.deck.set_button_image(12, IMG_FALLING_DOWN)?;
                }
            }

            13 => {
                if matches!(message.state, ButtonState::Up) {
                    state.deck.set_button_image(13, IMG_EYES)?;

                    state.matrix_animation = MatrixAnimation::Eyes;
                    state.start_matrix_animation();
                } else {
                    state.deck.set_button_image(13, IMG_EYES_DOWN)?;
                }
            }

            14 => {
                if matches!(message.state, ButtonState::Up) {
                    state.deck.set_button_image(14, IMG_CLOCK)?;

                    let animation = match state.matrix_animation {
                        MatrixAnimation::Time => MatrixAnimation::TimeNoClock,
                        _ => MatrixAnimation::Time,
                    };

                    state.matrix_animation = animation;
                    state.start_matrix_animation();
                } else {
                    state.deck.set_button_image(14, IMG_CLOCK_DOWN)?;
                }
            }

            _ => {}
        }
    }

    Ok(())
}

fn render_animation_items(state: &mut AppState) -> Result<()> {
    state.deck.set_button_image(14, IMG_CLOCK)?;
    state.deck.set_button_image(13, IMG_EYES)?;
    state.deck.set_button_image(12, IMG_FALLING)?;
    state.deck.set_button_image(11, IMG_BLOCKS)?;
    state.deck.set_button_image(10, IMG_SEQUENCE)?;

    Ok(())
}
