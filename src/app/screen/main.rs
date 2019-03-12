use alloc::boxed::Box;
use alloc::vec::Vec;
use orbclient::{Color, Renderer};
use orbfont::Font;
use uefi::status::Result;

use display::Display;
use key::Key;

use super::{Screen, SettingScreen};

pub struct MainScreen {
    entries: Vec<&'static str>,
    selected: usize,
}

impl MainScreen {
    pub fn new(mut selected: usize) -> Result<Box<Screen>> {
        let entries = vec![
            "Continue",
            "Boot Menu",
            "Settings",
        ];

        if selected >= entries.len() {
            selected = 0;
        }

        Ok(Box::new(MainScreen {
            entries: entries,
            selected: selected,
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
