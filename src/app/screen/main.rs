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

        let font_width = 8;
        let font_height = 16;
        let padding = 12;
        let margin = 8;

        let entry_height = padding + font_height + padding;

        let mut y = (display.height() as i32 - self.entries.len() as i32 * (entry_height + margin))/2;

        for (i, entry) in self.entries.iter().enumerate() {
            let entry_width = 200; //padding + entry.len() as i32 * font_width + padding;

            let mut x = (display.width() as i32 - entry_width)/2;

            let (fg, bg) = if i == self.selected {
                (
                    Color::rgb(0x2f, 0x2f, 0x2f),
                    Color::rgb(0xeb, 0xeb, 0xeb),
                )
            } else {
                (
                    Color::rgb(0xeb, 0xeb, 0xeb),
                    Color::rgb(0x13, 0x13, 0x13),
                )
            };

            if i == self.selected {
                display.rounded_rect(x - 2, y - 2, entry_width as u32 + 4, entry_height as u32 + 4, 8, true, Color::rgb(0x94, 0x94, 0x94));
                display.rounded_rect(x + 2, y + 2, entry_width as u32 - 4, entry_height as u32 - 4, 8, true, bg);
            } else {
                display.rect(x, y, entry_width as u32, entry_height as u32, bg);
            }

            x += padding;

            for c in entry.chars() {
                display.char(x, y + padding, c, fg);
                x += font_width;
            }

            y += entry_height + margin;
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
