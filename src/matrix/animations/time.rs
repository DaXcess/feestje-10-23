use super::{Animation, ImageBuffer};
use crate::{
    image,
    matrix::{self, iter::IterLooping},
};
use embedded_graphics::{
    image::ImageRaw,
    mono_font::{mapping::StrGlyphMapping, DecorationDimensions, MonoFont, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::Rectangle,
    text::{Alignment, Baseline, Text, TextStyle, TextStyleBuilder},
};
use image::RgbImage;
use std::time::{Duration, Instant};

const SEVENT_SEGMENT_FONT: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("../../../assets/font.raw"), 120),
    glyph_mapping: &StrGlyphMapping::new("0123456789 :", 0),
    character_size: Size::new(10, 18),
    character_spacing: 1,
    baseline: 18,
    underline: DecorationDimensions::default_underline(18),
    strikethrough: DecorationDimensions::default_strikethrough(18),
};

const TEXT_POSITION: Point = Point::new(31, 24);
const TEXT_STYLE: TextStyle = TextStyleBuilder::new()
    .baseline(Baseline::Bottom)
    .alignment(Alignment::Center)
    .build();

const CHARACTER_STYLE: MonoTextStyle<'_, Rgb888> =
    MonoTextStyle::new(&SEVENT_SEGMENT_FONT, Rgb888::WHITE);

pub struct TimeAnimation {
    last_frame: Instant,
    frame: usize,

    disable_clock: bool,
    last_hour: u8,
    last_minute: u8,
    last_colon: bool,

    table: Vec<Rgb888>,
    last_image: Option<ImageBuffer>,
}

impl TimeAnimation {
    pub fn new(disable_clock: bool) -> Self {
        let table = matrix::color_utils::generate_table();

        Self {
            last_frame: Instant::now(),
            frame: 0,

            disable_clock,
            last_hour: 0,
            last_minute: 0,
            last_colon: true,

            table,
            last_image: None,
        }
    }
}

impl Animation for TimeAnimation {
    fn should_execute(&self) -> bool {
        self.last_frame.elapsed() > Duration::from_millis(10)
    }

    fn next_frame(&mut self) -> Option<RgbImage> {
        let mut result = self
            .last_image
            .clone()
            .unwrap_or(ImageBuffer(vec![0; 3 * 64 * 32]));

        // Render colored borders

        // X: 0----63, Y: 0
        let area = Rectangle::new(Point::new(0, 0), Size::new(64, 1));
        result
            .draw_iter(
                area.points()
                    .zip(self.table.iter_looping().skip(190 - self.frame))
                    .map(|(pos, color)| Pixel(pos, color)),
            )
            .ok()?;

        // Y: 1----31, X: 63
        let area = Rectangle::new(Point::new(63, 1), Size::new(1, 31));
        result
            .draw_iter(
                area.points()
                    .zip(
                        self.table
                            .iter_looping()
                            .skip((64 + 190 - self.frame) % 190),
                    )
                    .map(|(pos, color)| Pixel(pos, color)),
            )
            .ok()?;

        // X: 62----0, Y: 31
        let area = Rectangle::new(Point::new(0, 31), Size::new(63, 1));
        result
            .draw_iter(
                area.points()
                    .zip(
                        self.table
                            .iter_looping()
                            .rev()
                            .skip((32 + self.frame) % 190),
                    )
                    .map(|(pos, color)| Pixel(pos, color)),
            )
            .ok()?;

        // Y: 30----1, X: 0
        let area = Rectangle::new(Point::new(0, 1), Size::new(1, 30));
        result
            .draw_iter(
                area.points()
                    .zip(self.table.iter_looping().skip(self.frame))
                    .map(|(pos, color)| Pixel(pos, color)),
            )
            .ok()?;

        // X: 1----62, Y: 1
        let area = Rectangle::new(Point::new(1, 1), Size::new(62, 1));
        result
            .draw_iter(
                area.points()
                    .zip(self.table.iter_looping().skip(189 - (self.frame % 189)))
                    .map(|(pos, color)| Pixel(pos, color)),
            )
            .ok()?;

        // Y: 2----30, X: 62
        let area = Rectangle::new(Point::new(62, 2), Size::new(1, 30));
        result
            .draw_iter(
                area.points()
                    .zip(
                        self.table
                            .iter_looping()
                            .skip((64 + 189 - (self.frame % 189)) % 190),
                    )
                    .map(|(pos, color)| Pixel(pos, color)),
            )
            .ok()?;

        // X: 61----1, Y: 30
        let area = Rectangle::new(Point::new(1, 30), Size::new(61, 1));
        result
            .draw_iter(
                area.points()
                    .zip(
                        self.table
                            .iter_looping()
                            .rev()
                            .skip((31 + self.frame) % 190),
                    )
                    .map(|(pos, color)| Pixel(pos, color)),
            )
            .ok()?;

        // Y: 29----2, X: 1
        let area = Rectangle::new(Point::new(1, 2), Size::new(1, 28));
        result
            .draw_iter(
                area.points()
                    .zip(self.table.iter_looping().skip(self.frame % 189))
                    .map(|(pos, color)| Pixel(pos, color)),
            )
            .ok()?;

        if self.disable_clock {
            self.frame = (self.frame + 1) % 190;

            let image = RgbImage::from_raw(64, 32, result.0.clone())?;
            return Some(image);
        }

        // Render time

        let local = time::OffsetDateTime::now_utc();
        let (h, m, ms) = (local.hour(), local.minute(), local.millisecond());

        // Timezone shenanigans
        let h = if (1..10).contains(&h) { h + 1 } else { h + 2 } % 24;

        let colon = ms > 500;

        if h == self.last_hour && m == self.last_minute && colon == self.last_colon {
            self.frame = (self.frame + 1) % 190;

            let image = RgbImage::from_raw(64, 32, result.0.clone())?;
            return Some(image);
        }

        self.last_hour = h;
        self.last_minute = m;
        self.last_colon = colon;

        let area = Rectangle::new(Point::new(2, 2), Size::new(60, 28));
        result
            .draw_iter(area.points().map(|pos| Pixel(pos, Rgb888::BLACK)))
            .ok()?;

        Text::with_text_style(
            &format!("{h:0>2}{}{m:0>2}", if colon { ":" } else { " " }),
            TEXT_POSITION,
            CHARACTER_STYLE,
            TEXT_STYLE,
        )
        .draw(&mut result)
        .ok()?;

        self.frame = (self.frame + 1) % 190;

        self.last_image = Some(result.clone());
        let image = RgbImage::from_raw(64, 32, result.0)?;

        Some(image)
    }

    fn reload(&mut self) {
        self.table = matrix::color_utils::generate_table();
    }
}
