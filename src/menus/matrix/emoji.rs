use crate::{image::ImageSourceType, state::AppState};
use anyhow::Result;
use image::{Rgb, RgbImage};
use streamdeck_hid_rs::ButtonState;

const IMG_BACK: ImageSourceType = ImageSourceType::Jpeg(include_bytes!("../../../images/back.jpg"));
const IMG_BACK_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../../images/back_down.jpg"));

const IMG_CLEAR: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../../images/clear.jpg"));
const IMG_CLEAR_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../../images/clear_down.jpg"));

const IMG_NEXT: ImageSourceType = ImageSourceType::Jpeg(include_bytes!("../../../images/next.jpg"));
const IMG_NEXT_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../../images/next_down.jpg"));

const IMG_PREV: ImageSourceType = ImageSourceType::Jpeg(include_bytes!("../../../images/prev.jpg"));
const IMG_PREV_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../../images/prev_down.jpg"));

// Single emoji layout:

// |-------|-------|-------|-------|-------|
// |       |       |       |       |       |
// | BACK  |   0   |   1   |   2   | CLEAR |
// |       |       |       |       |       |
// |-------|-------|-------|-------|-------|
// |       |       |       |       |       |
// | PREVP |   3   |   4   |   5   | NEXTP |
// |       |       |       |       |       |
// |-------|-------|-------|-------|-------|
// |       |       |       |       |       |
// |   6   |   7   |   8   |   9   |  1 0  |
// |       |       |       |       |       |
// |-------|-------|-------|-------|-------|

const EMOJIS: &[&str] = &[
    "backhand-index-pointing-left",
    "index-pointing-at-the-viewer",
    "backhand-index-pointing-right",
    "face-with-tears-of-joy",
    "middle-finger",
    "face-with-raised-eyebrow",
    "ghost",
    "pile-of-poo",
    "clown-face",
    "face-screaming-in-fear",
    "waving-hand",
    "ok-hand",
    "pinched-fingers",
    "pinching-hand",
    "thumbs-up",
    "thumbs-down",
    "clapping-hands",
    "handshake",
    "palms-up-together",
    "face-vomiting",
    "smirking-face",
    "neutral-face",
    "exploding-head",
    "skull",
    "sweat-droplets",
    "nose",
    "rocket",
    "airplane",
    "hotel",
    "trophy",
    "party-popper",
    "fire",
    "snowman-without-snow",
    "hourglass-not-done",
    "beer-mug",
    "clinking-beer-mugs",
    "egg",
    "hatching-chick",
    "baby-chick",
    "poultry-leg",
    "sleeping-face",
];

const DECK_MAPPING: &[u16] = &[0, 0, 1, 2, 0, 0, 3, 4, 5, 0, 6, 7, 8, 9, 10];
const EMOJI_MAPPING: &[u8] = &[1, 2, 3, 6, 7, 8, 10, 11, 12, 13, 14];

const PAGE_SIZE: usize = 11;

pub fn launch(state: &mut AppState) -> Result<()> {
    let mut page = 0;

    state.deck.clear()?;

    state.deck.set_button_image(0, IMG_BACK)?;
    state.deck.set_button_image(4, IMG_CLEAR)?;
    state.deck.set_button_image(5, IMG_PREV)?;
    state.deck.set_button_image(9, IMG_NEXT)?;

    render_emojis(state, page)?;

    loop {
        let message = state.deck.next_btn_event()?;
        let id = message.button_id as u8;

        match id {
            0 => {
                if matches!(message.state, ButtonState::Up) {
                    break;
                } else {
                    state.deck.set_button_image(id, IMG_BACK_DOWN)?;
                }
            }
            4 => {
                if matches!(message.state, ButtonState::Up) {
                    // Clear image & update matrix

                    state.matrix.clear()?;
                    state.deck.set_button_image(id, IMG_CLEAR)?;
                } else {
                    state.deck.set_button_image(id, IMG_CLEAR_DOWN)?;
                }
            }
            5 => {
                if matches!(message.state, ButtonState::Up) {
                    // Go back one page, unless we are at start

                    state.deck.set_button_image(id, IMG_PREV)?;

                    let next_page = std::cmp::max(0i32, page as i32 - 1) as u16;
                    if next_page != page {
                        page = next_page;

                        render_emojis(state, page)?;
                    }
                } else {
                    state.deck.set_button_image(id, IMG_PREV_DOWN)?;
                }
            }
            9 => {
                if matches!(message.state, ButtonState::Up) {
                    // Advance page, unless this is the last page

                    state.deck.set_button_image(id, IMG_NEXT)?;

                    let next_page =
                        std::cmp::min(EMOJIS.len() / PAGE_SIZE, page as usize + 1) as u16;
                    if next_page != page {
                        page = next_page;

                        render_emojis(state, page)?;
                    }
                } else {
                    state.deck.set_button_image(id, IMG_NEXT_DOWN)?;
                }
            }
            _ => {
                // Else set matrix emoji image

                if matches!(message.state, ButtonState::Down) {
                    continue;
                }

                let index = DECK_MAPPING[id as usize] + PAGE_SIZE as u16 * page;
                let Some(emoji_name) = EMOJIS.get(index as usize) else {
                    continue;
                };

                let Some(data) = state.emojis.get_emoji(emoji_name) else {
                    continue;
                };

                let mut image = RgbImage::new(64, 32);
                for y in 0..32usize {
                    for x in 0..32usize {
                        image.put_pixel(
                            x as u32 + 16,
                            y as u32,
                            Rgb([
                                data[(y * 32 + x) * 3],
                                data[(y * 32 + x) * 3 + 1],
                                data[(y * 32 + x) * 3 + 2],
                            ]),
                        );
                    }
                }

                state.matrix.set_image(image)?;
            }
        }
    }

    Ok(())
}

const EMPTY: &[u8] = &[0; 32 * 32 * 3];

fn render_emojis(state: &AppState, page: u16) -> Result<()> {
    for (i, emoji) in EMOJIS
        .iter()
        .skip(page as usize * PAGE_SIZE)
        .take(PAGE_SIZE)
        .enumerate()
    {
        let mut image = RgbImage::new(72, 72);
        let emoji = state.emojis.get_emoji(emoji).unwrap_or(EMPTY);

        for y in 0..32usize {
            for x in 0..32usize {
                image.put_pixel(
                    x as u32 + 20,
                    y as u32 + 20,
                    Rgb([
                        emoji[(y * 32 + x) * 3],
                        emoji[(y * 32 + x) * 3 + 1],
                        emoji[(y * 32 + x) * 3 + 2],
                    ]),
                );
            }
        }

        state
            .deck
            .set_button_image(EMOJI_MAPPING[i], ImageSourceType::Rgb(image))?;
    }

    // The above code can take a bit longer than desired,
    //  so we flush the button events in case of additional
    //  button presses during the rendering
    state.deck.flush_btn_events()?;

    Ok(())
}
