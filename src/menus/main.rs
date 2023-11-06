use std::{
    fs::File,
    io::Read,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};

use anyhow::Result;
use streamdeck_hid_rs::ButtonState;

use crate::{image::ImageSourceType, render::render_text, AppState, Deck};

const IMG_CAMERA: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/camera.jpg"));
const IMG_CAMERA_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/camera_hold.jpg"));

const IMG_MATRIX: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/matrix.jpg"));
const IMG_MATRIX_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/matrix_down.jpg"));

const IMG_GAME: ImageSourceType = ImageSourceType::Jpeg(include_bytes!("../../images/game.jpg"));
const IMG_GAME_DOWN: ImageSourceType =
    ImageSourceType::Jpeg(include_bytes!("../../images/game_down.jpg"));

pub fn launch(state: &mut AppState) -> Result<()> {
    state.deck.clear()?;
    state.deck.set_button_image(0, IMG_CAMERA)?;
    state.deck.set_button_image(1, IMG_MATRIX)?;
    state.deck.set_button_image(2, IMG_GAME)?;

    let mut task = MainMenuTask::new(state.deck.device());
    task.start();

    loop {
        let message = state.deck.next_btn_event()?;
        let id = message.button_id as u8;

        match id {
            0 => {
                if matches!(message.state, ButtonState::Up) {
                    task.stop();

                    super::camera::launch(state)?;

                    state.deck.clear()?;
                    state.deck.set_button_image(0, IMG_CAMERA)?;
                    state.deck.set_button_image(1, IMG_MATRIX)?;
                    state.deck.set_button_image(2, IMG_GAME)?;

                    task.start();
                } else {
                    state.deck.set_button_image(id, IMG_CAMERA_DOWN)?;
                }
            }

            1 => {
                if matches!(message.state, ButtonState::Up) {
                    task.stop();

                    super::matrix::launch(state)?;

                    state.deck.clear()?;
                    state.deck.set_button_image(0, IMG_CAMERA)?;
                    state.deck.set_button_image(1, IMG_MATRIX)?;
                    state.deck.set_button_image(2, IMG_GAME)?;
                } else {
                    state.deck.set_button_image(id, IMG_MATRIX_DOWN)?;
                }
            }

            2 => {
                if matches!(message.state, ButtonState::Up) {
                    task.stop();

                    super::games::launch(state)?;

                    state.deck.clear()?;
                    state.deck.set_button_image(0, IMG_CAMERA)?;
                    state.deck.set_button_image(1, IMG_MATRIX)?;
                    state.deck.set_button_image(2, IMG_GAME)?;

                    task.start();
                } else {
                    state.deck.set_button_image(id, IMG_GAME_DOWN)?;
                }
            }

            _ => {}
        }
    }
}

struct MainMenuTask {
    signal: Arc<AtomicBool>,
    device: Deck,
}

impl MainMenuTask {
    pub fn new(device: Deck) -> Self {
        Self {
            signal: Arc::new(AtomicBool::new(false)),
            device,
        }
    }

    pub fn start(&mut self) {
        self.signal = Arc::new(AtomicBool::new(false));
        self.signal.store(false, Ordering::Relaxed);

        std::thread::spawn({
            let signal = self.signal.clone();
            let device = self.device.clone();

            move || -> Result<()> {
                let mut last_h = 0;
                let mut last_m = 0;
                let mut last_t = 0;

                while !signal.load(Ordering::Relaxed) {
                    let local = time::OffsetDateTime::now_utc();
                    let (h, m) = (local.hour(), local.minute());

                    if let Ok(mut file) = File::open("/sys/class/thermal/thermal_zone0/temp") {
                        let mut temp = String::new();
                        if file.read_to_string(&mut temp).is_ok() {
                            let t: u8 = (temp.trim().parse::<u32>()? / 1000) as u8;

                            if t != last_t {
                                last_t = t;

                                let t = render_text(format!("{t}C"), 32)
                                    .expect("cannot convert to text");

                                device
                                    .set_button_image(10, ImageSourceType::Rgb(t))
                                    .expect("failed to set button image");
                            }
                        }
                    }

                    // Timezone shenanigans
                    let h = if (1..10).contains(&h) { h + 1 } else { h + 2 } % 24;

                    if h == last_h && m == last_m {
                        sleep(Duration::from_secs(1));
                        continue;
                    }

                    last_h = h;
                    last_m = m;

                    let h = render_text(format!("{h:0>2}"), 48).expect("cannot convert to text");
                    let m = render_text(format!("{m:0>2}"), 48).expect("cannot convert to text");

                    device
                        .set_button_image(13, ImageSourceType::Rgb(h))
                        .expect("failed to set button image");

                    device
                        .set_button_image(14, ImageSourceType::Rgb(m))
                        .expect("failed to set button image");

                    sleep(Duration::from_secs(1));
                }

                Ok(())
            }
        });
    }

    pub fn stop(&self) {
        self.signal.store(true, Ordering::Relaxed);
    }
}
