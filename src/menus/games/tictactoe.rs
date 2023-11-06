use std::time::Duration;

use crate::{
    deck::Deck,
    image::ImageSourceType,
    matrix::animations::{TicTacToeAnimation, Winner},
    AppState,
};
use anyhow::Result;
use image::RgbImage;
use streamdeck_hid_rs::ButtonState;

type Board = [State; 9];

const WHITE: image::Rgb<u8> = image::Rgb([255, 255, 255]);
const BUTTON_MAP: [usize; 15] = [
    0xff, 0xff, 0x00, 0x01, 0x02, 0xff, 0xff, 0x03, 0x04, 0x05, 0xff, 0xff, 0x06, 0x07, 0x08,
];

const IMG_BACK: ImageSourceType = ImageSourceType::Jpeg(include_bytes!("../../../images/back.jpg"));
const IMG_ARROWS_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../../images/arrows_down.jpg"));
const IMG_ARROWS_UP: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../../images/arrows_up.jpg"));

#[derive(Clone, Copy, PartialEq)]
enum State {
    Blank,
    X,
    O,
}

enum Turn {
    X,
    O,
}

impl Turn {
    fn state(&self) -> State {
        match self {
            Self::X => State::X,
            Self::O => State::O,
        }
    }

    fn flip(&mut self) {
        match self {
            Self::X => *self = Self::O,
            Self::O => *self = Self::X,
        }
    }
}

pub fn launch(state: &mut AppState) -> Result<()> {
    let mut board: Board = [State::Blank; 9];
    let mut turn = Turn::X;

    state.deck.clear()?;
    state.deck.set_button_image(5, IMG_BACK)?;
    state.deck.set_button_image(1, IMG_ARROWS_DOWN)?;
    state.deck.set_button_image(11, IMG_ARROWS_UP)?;

    state.matrix.set_image(render_matrix(&board))?;
    render_streamdeck(&state.deck, &board, &turn)?;

    loop {
        let btn_event = state.deck.next_btn_event()?;
        if btn_event.button_id == 5 && matches!(btn_event.state, ButtonState::Up) {
            // Back button pressed

            break;
        }

        let board_idx = BUTTON_MAP[btn_event.button_id as usize];
        if board_idx == 0xff {
            // Invalid button pressed

            continue;
        }

        if board[board_idx] != State::Blank {
            continue;
        }

        board[board_idx] = turn.state();

        let x = board_idx % 3;
        let y = board_idx / 3;
        if check_win(&board, &turn, x, y) {
            render_streamdeck(&state.deck, &board, &turn)?;

            state
                .matrix
                .set_animation(Box::new(TicTacToeAnimation::new(match turn {
                    Turn::O => Winner::O,
                    Turn::X => Winner::X,
                })))?;
            std::thread::sleep(Duration::from_secs(3));

            break;
        }

        // Check if there are no moves left
        if board.iter().fold(0, |acc, v| {
            acc + if matches!(v, State::Blank) { 1 } else { 0 }
        }) == 0
        {
            render_streamdeck(&state.deck, &board, &turn)?;

            state
                .matrix
                .set_animation(Box::new(TicTacToeAnimation::new(Winner::Draw)))?;
            std::thread::sleep(Duration::from_secs(3));

            break;
        }

        // Must flip after win check because doing so before will make it so nobody can ever win
        turn.flip();

        state.matrix.set_image(render_matrix(&board))?;
        render_streamdeck(&state.deck, &board, &turn)?;
    }

    Ok(())
}

fn render_matrix(board: &Board) -> RgbImage {
    const X_OFFSET: u32 = 16;

    let mut image = RgbImage::new(32 + X_OFFSET * 2, 32);

    // Vertical lines
    for x in 0..=1 {
        for y in 0..32 {
            image.put_pixel(X_OFFSET + if x == 0 { 10 } else { 21 }, y, WHITE);
        }
    }

    // Horizontal lines
    for y in 0..=1 {
        for x in 0..32 {
            image.put_pixel(X_OFFSET + x, if y == 0 { 10 } else { 21 }, WHITE);
        }
    }

    // Draw X, algorithmic approach
    let draw_x = |image: &mut RgbImage, slot: u32| {
        let x_off = X_OFFSET + (slot % 3) * 11 + 1;
        let y_off = (slot / 3) * 11 + 1;

        for y in 0..4 {
            let skip = 6 - y * 2;
            let x = x_off + y;

            image.put_pixel(x, y_off + y, WHITE);
            image.put_pixel(x + skip + 1, y_off + y, WHITE);
        }

        for y in 4..8 {
            let skip = (y - 4) * 2;
            let x = x_off + (7 - y);

            image.put_pixel(x, y_off + y, WHITE);
            image.put_pixel(x + skip + 1, y_off + y, WHITE);
        }
    };

    // Draw O, no algorithmic approach, just spam image.put_pixel
    let draw_o = |image: &mut RgbImage, slot: u32| {
        let x = X_OFFSET + (slot % 3) * 11 + 1;
        let y = (slot / 3) * 11 + 1;

        image.put_pixel(x + 2, y, WHITE);
        image.put_pixel(x + 3, y, WHITE);
        image.put_pixel(x + 4, y, WHITE);
        image.put_pixel(x + 5, y, WHITE);

        image.put_pixel(x + 1, y + 1, WHITE);
        image.put_pixel(x + 6, y + 1, WHITE);

        image.put_pixel(x, y + 2, WHITE);
        image.put_pixel(x + 7, y + 2, WHITE);
        image.put_pixel(x, y + 3, WHITE);
        image.put_pixel(x + 7, y + 3, WHITE);
        image.put_pixel(x, y + 4, WHITE);
        image.put_pixel(x + 7, y + 4, WHITE);
        image.put_pixel(x, y + 5, WHITE);
        image.put_pixel(x + 7, y + 5, WHITE);

        image.put_pixel(x + 1, y + 6, WHITE);
        image.put_pixel(x + 6, y + 6, WHITE);

        image.put_pixel(x + 2, y + 7, WHITE);
        image.put_pixel(x + 3, y + 7, WHITE);
        image.put_pixel(x + 4, y + 7, WHITE);
        image.put_pixel(x + 5, y + 7, WHITE);
    };

    for (i, state) in board.iter().enumerate() {
        match state {
            State::O => draw_o(&mut image, i as u32),
            State::X => draw_x(&mut image, i as u32),
            State::Blank => {}
        }
    }

    image
}

fn render_streamdeck(deck: &Deck, board: &Board, turn: &Turn) -> Result<()> {
    let x = crate::render::render_text("X", 64)?;
    let o = crate::render::render_text("O", 64)?;
    let d = crate::render::render_text(".", 64)?;

    let turn_img = match turn {
        Turn::X => &x,
        Turn::O => &o,
    };

    deck.set_button_image(6, ImageSourceType::Rgb(turn_img.clone()))?;

    for (i, state) in board.iter().enumerate() {
        let deck_idx = (i % 3 + 2) + (i / 3) * 5;

        match state {
            State::X => deck.set_button_image(deck_idx as u8, ImageSourceType::Rgb(x.clone()))?,
            State::O => deck.set_button_image(deck_idx as u8, ImageSourceType::Rgb(o.clone()))?,
            State::Blank => {
                deck.set_button_image(deck_idx as u8, ImageSourceType::Rgb(d.clone()))?
            }
        }
    }

    Ok(())
}

/// Called after a move has been performed
///
/// `turn` will be set to the person who has performed the last move
fn check_win(board: &Board, turn: &Turn, x: usize, y: usize) -> bool {
    // Stolen from https://stackoverflow.com/questions/1056316/algorithm-for-determining-tic-tac-toe-game-over

    // Col
    for i in 0..3 {
        if board[i + y * 3] != turn.state() {
            break;
        }

        if i == 2 {
            return true;
        }
    }

    // Row
    for i in 0..3 {
        if board[i * 3 + x] != turn.state() {
            break;
        }

        if i == 2 {
            return true;
        }
    }

    // Diagonal
    if x == y {
        for i in 0..3 {
            if board[i * 3 + i] != turn.state() {
                break;
            }

            if i == 2 {
                return true;
            }
        }
    }

    // Anti-Diagonal
    if x + y == 2 {
        for i in 0..3 {
            if board[(2 - i) * 3 + i] != turn.state() {
                break;
            }

            if i == 2 {
                return true;
            }
        }
    }

    false
}
