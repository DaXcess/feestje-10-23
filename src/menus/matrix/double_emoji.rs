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

const IMG_SELECT: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../../images/select.jpg"));
const IMG_SELECT_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../../images/select_down.jpg"));
const IMG_SELECT_X: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../../images/select-x.jpg"));
const IMG_SELECT_X_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../../images/select-x_down.jpg"));

// Double emoji layout:

// |-------|-------|-------|-------|-------|
// |       |       |       |       |       |
// | BACK  |       |       |       | CLEAR |
// |       |       |       |       |       |
// |-------|-------|-------|-------|-------|
// |       |       |       |       |       |
// | PREVP |       |       |       | NEXTP |
// |       |       |       |       |       |
// |-------|-------|-------|-------|-------|
// | SEL & |       |       |       | SEL & |
// | CLEAR |       |       |       | CLEAR |
// | LEFT  |       |       |       | RIGHT |
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

const DECK_MAPPING: [u16; 15] = [0, 0, 1, 2, 0, 0, 3, 4, 5, 0, 0, 6, 7, 8, 0];
const EMOJI_MAPPING: [u8; 9] = [1, 2, 3, 6, 7, 8, 11, 12, 13];

const PAGE_SIZE: usize = 9;

enum Selection {
    None,
    Left,
    Right,
}

impl Selection {
    fn matches(&self, id: u8) -> bool {
        matches!((self, id), (Selection::Left, 10) | (Selection::Right, 14))
    }

    fn offset(&self) -> u32 {
        match self {
            Selection::Left => 0,
            Selection::Right => 32,

            Selection::None => 0,
        }
    }

    fn is_none(&self) -> bool {
        matches!(self, Selection::None)
    }
}

pub fn launch(state: &mut AppState) -> Result<()> {
    let mut page = 0;
    let mut img = RgbImage::new(64, 32);
    let mut selection = Selection::None;

    state.deck.clear()?;

    state.deck.set_button_image(0, IMG_BACK)?;
    state.deck.set_button_image(4, IMG_CLEAR)?;
    state.deck.set_button_image(5, IMG_PREV)?;
    state.deck.set_button_image(9, IMG_NEXT)?;
    state.deck.set_button_image(10, IMG_SELECT)?;
    state.deck.set_button_image(14, IMG_SELECT)?;

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
                    // Clear image & matrix

                    if matches!(selection, Selection::Left | Selection::Right) {
                        for y in 0..32 {
                            for x in selection.offset()..selection.offset() + 32 {
                                img.put_pixel(x, y, Rgb([0, 0, 0]));
                            }
                        }

                        state.matrix.set_image(img.clone())?;
                    }

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
            10 | 14 => {
                if matches!(message.state, ButtonState::Up) {
                    if id == 10 {
                        selection = Selection::Left;
                    } else {
                        selection = Selection::Right;
                    }

                    state.deck.set_button_image(
                        10,
                        if selection.matches(10) {
                            IMG_SELECT_X
                        } else {
                            IMG_SELECT
                        },
                    )?;

                    state.deck.set_button_image(
                        14,
                        if selection.matches(14) {
                            IMG_SELECT_X
                        } else {
                            IMG_SELECT
                        },
                    )?;
                } else {
                    // Render down

                    state.deck.set_button_image(
                        id,
                        if selection.matches(id) {
                            IMG_SELECT_X_DOWN
                        } else {
                            IMG_SELECT_DOWN
                        },
                    )?;
                }
            }
            _ => {
                // Else set matrix emoji image

                if selection.is_none() {
                    continue;
                }

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

                for y in 0..32usize {
                    for x in 0..32 {
                        img.put_pixel(
                            x as u32 + selection.offset(),
                            y as u32,
                            Rgb([
                                data[(y * 32 + x) * 3],
                                data[(y * 32 + x) * 3 + 1],
                                data[(y * 32 + x) * 3 + 2],
                            ]),
                        );
                    }
                }

                state.matrix.set_image(img.clone())?;
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
