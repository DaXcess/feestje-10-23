use std::{
    ops::Deref,
    sync::{
        mpsc::{Receiver, Sender, TryRecvError},
        Arc,
    },
};

use anyhow::{anyhow, Result};
use hidapi::HidApi;
use streamdeck_hid_rs::{ButtonEvent, ButtonState, StreamDeckDevice};

use crate::image::{self, ImageSourceType};

pub struct DeckReceiver(Deck, Receiver<ButtonEvent>);

impl Deref for DeckReceiver {
    type Target = Deck;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct Deck {
    device: Arc<StreamDeckDevice<HidApi>>,
    tx: Sender<ButtonEvent>,
}

impl Deref for Deck {
    type Target = Arc<StreamDeckDevice<HidApi>>;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl Deck {
    pub fn open() -> Result<DeckReceiver> {
        let hidapi = HidApi::new()?;
        let device = Arc::new(
            StreamDeckDevice::open_first_device(&hidapi)
                .map_err(|_| anyhow!("Failed to open Stream Deck device"))?,
        );

        let (tx, rx) = std::sync::mpsc::channel();

        Ok(DeckReceiver(Self { device, tx }, rx))
    }

    pub fn clear(&self) -> Result<()> {
        let (width, height) = self.device_type.button_image_size();
        let image = image::RgbImage::new(width, height);

        for idx in 0..self.device_type.total_num_buttons() {
            self.device
                .set_button_image(idx as u8, &image)
                .map_err(|why| anyhow!("Failed to set button image: {why:?}"))?;
        }

        Ok(())
    }

    pub fn set_button_image(&self, index: u8, image: ImageSourceType) -> Result<()> {
        let image = &image.to_rgb(72, 72)?;

        self.device
            .set_button_image(index, image)
            .map_err(|why| anyhow!("Failed to set button image: {why:?}"))
    }

    pub fn set_fullscreen_image(&self, image: ImageSourceType) -> Result<()> {
        let split = image::split_raw_full_image(&image.to_raw()?)
            .ok_or_else(|| anyhow!("Invalid image data"))?;

        for (idx, buffer) in split.into_iter().enumerate() {
            self.device
                .set_button_image(idx as u8, &buffer)
                .map_err(|why| anyhow!("Failed to set button image: {why:?}"))?;
        }

        Ok(())
    }

    pub fn start_event_loop(&self) {
        std::thread::spawn({
            let device = self.device.clone();
            let tx = self.tx.clone();

            move || {
                device
                    .on_button_events(|event| {
                        tx.send(event).expect("channel closed");
                    })
                    .ok();

                tx.send(ButtonEvent {
                    button_id: 0xffffffff,
                    state: ButtonState::Down,
                })
            }
        });
    }
}

impl DeckReceiver {
    pub fn next_btn_event(&self) -> Result<ButtonEvent> {
        let event = self.1.recv()?;

        if event.button_id == 0xffffffff {
            return Err(anyhow!("HID communication failure"));
        }

        Ok(event)
    }

    pub fn wait_for_any_press(&self) -> Result<()> {
        loop {
            let event = self.next_btn_event()?;

            if matches!(event.state, ButtonState::Up) {
                break;
            }
        }

        Ok(())
    }

    pub fn flush_btn_events(&self) -> Result<()> {
        loop {
            match self.1.try_recv() {
                Err(TryRecvError::Empty) => {
                    return Ok(());
                }

                Err(TryRecvError::Disconnected) => {
                    Err(TryRecvError::Disconnected)?;
                }

                Ok(_) => {}
            }
        }
    }

    pub fn device(&self) -> Deck {
        self.0.clone()
    }
}
