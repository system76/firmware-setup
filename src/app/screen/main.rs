use orbclient::{Color, Renderer};
use orbfont::Font;
use uefi::status::Result;

use crate::display::Display;
use crate::key::Key;

use super::{Screen, BootScreen, SettingScreen};

pub struct MainScreen {
    entries: Vec<&'static str>,
    row: usize,
}

impl MainScreen {
    pub fn new(mut row: usize) -> Result<Box<Screen>> {
        let entries = vec![
            "Continue",
            "Boot Menu",
            "Settings",
        ];

        if row >= entries.len() {
            row = 0;
        }

        Ok(Box::new(MainScreen {
            entries: entries,
            row: row,
        }))
    }
}

impl Screen for MainScreen {
    fn draw(&self, display: &mut Display, font: &Font) {
        let font_height = 24;
        let padding = 16;
        let margin = 12;

        let entry_height = padding + font_height + padding;

        let mut y = (display.height() as i32 - self.entries.len() as i32 * (entry_height + margin))/2;

        for (i, entry) in self.entries.iter().enumerate() {
            let entry_width = 200;

            let mut x = (display.width() as i32 - entry_width)/2;

            let (fg, bg) = if i == self.row {
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

            if i == self.row {
                display.rounded_rect(x - 2, y - 2, entry_width as u32 + 4, entry_height as u32 + 4, 8, true, Color::rgb(0x94, 0x94, 0x94));
                display.rounded_rect(x + 2, y + 2, entry_width as u32 - 4, entry_height as u32 - 4, 6, true, bg);
            } else {
                display.rect(x, y, entry_width as u32, entry_height as u32, bg);
            }

            x += padding;

            font.render(entry, font_height as f32).draw(display, x, y + padding, fg);

            y += entry_height + margin;
        }
    }

    fn key(mut self: Box<Self>, key: Key) -> Result<Option<Box<Screen>>> {
        match key {
            Key::Up if self.row > 0 => {
                self.row -= 1;
            },
            Key::Down if self.row + 1 < self.entries.len() => {
                self.row += 1;
            },
            Key::Enter => match self.row {
                1 => return BootScreen::new().map(Some),
                2 => return SettingScreen::new().map(Some),
                _ => (),
            },
            Key::Escape => return Ok(None),
            _ => (),
        }

        Ok(Some(self))
    }
}
