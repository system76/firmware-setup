use alloc::boxed::Box;
use alloc::vec::Vec;
use orbclient::{Color, Renderer};
use uefi::status::Result;

use display::Display;
use key::Key;

use super::{Screen, SettingScreen};

pub struct MainScreen {
    entries: Vec<&'static str>,
    selected: usize,
}

impl MainScreen {
    pub fn new() -> Result<Box<Screen>> {
        Ok(Box::new(MainScreen {
            entries: vec![
                "Continue",
                "Boot Menu",
                "Settings",
            ],
            selected: 0,
        }))
    }
}

impl Screen for MainScreen {
    fn draw(&self, display: &mut Display) {
        let gray = Color::rgb(0x41, 0x3e, 0x3c);
        let black = Color::rgb(0, 0, 0);
        let white = Color::rgb(0xFF, 0xFF, 0xFF);

        let mut y = (display.height() as i32 - self.entries.len() as i32 * 24)/2;

        for (i, entry) in self.entries.iter().enumerate() {
            let (fg, bg) = if i == self.selected {
                (black, white)
            } else {
                (white, gray)
            };

            let mut x = (display.width() as i32 - entry.len() as i32 * 8)/2;
            for c in entry.chars() {
                display.rect(x, y, 8, 16, bg);
                display.char(x, y, c, fg);
                x += 8;
            }

            y += 24;
        }
    }

    fn key(mut self: Box<Self>, key: Key) -> Result<Option<Box<Screen>>> {
        match key {
            Key::Up if self.selected > 0 => {
                self.selected -= 1;
            },
            Key::Down if self.selected + 1 < self.entries.len() => {
                self.selected += 1;
            },
            Key::Enter => match self.selected {
                2 => return SettingScreen::new().map(Some),
                _ => (),
            },
            Key::Escape => return Ok(None),
            _ => (),
        }

        Ok(Some(self))
    }
}
