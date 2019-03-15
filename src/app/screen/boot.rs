use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::{mem, slice};
use orbclient::{Color, Renderer};
use orbfont::Font;
use uefi::status::Result;

use crate::display::Display;
use crate::key::Key;
use crate::string::nstr;
use crate::vars;

use super::{Screen, MainScreen};

#[allow(non_snake_case)]
#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct LoadOption {
    pub Attributes: u32,
    pub FilePathListLength: u16,
}

impl LoadOption {
    pub unsafe fn description(&self) -> &[u16] {
        let ptr = (self as *const LoadOption as usize + mem::size_of::<LoadOption>()) as *const u16;
        let mut len = 0;
        while len < 2048 {
            if *ptr.add(len) == 0 {
                break;
            }
            len += 1;
        }
        slice::from_raw_parts(ptr, len)
    }
}

pub struct BootScreen {
    entries: Vec<(String, String)>,
    row: usize,
}

impl BootScreen {
    pub fn new() -> Result<Box<Screen>> {
        let mut entries = Vec::new();

        let boot_order = vars::get_boot_order()?;
        for boot_num in boot_order {
            let boot_item_raw = vars::get_boot_item(boot_num)?;
            unsafe {
                let boot_item = &*(boot_item_raw.as_ptr() as *const LoadOption);
                entries.push((
                    format!("Boot{:>04X}", boot_num),
                    nstr(boot_item.description().as_ptr())
                ));
            }
        }

        Ok(Box::new(BootScreen {
            entries: entries,
            row: 0,
        }))
    }
}

impl Screen for BootScreen {
    fn draw(&self, display: &mut Display, font: &Font) {
        let font_height = 24;
        let padding = 16;
        let margin = 12;

        let entry_height = padding + font_height + padding;

        let mut y = (display.height() as i32 - self.entries.len() as i32 * (entry_height + margin))/2;

        for (i, entry) in self.entries.iter().enumerate() {
            let entry_width = 320;
            let form_width = entry_width + margin + entry_width;

            let mut x = (display.width() as i32 - form_width)/2;

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

            font.render(&entry.0, font_height as f32).draw(display, x + padding, y + padding, fg);

            x += entry_width + margin;

            display.rect(x, y, entry_width as u32, entry_height as u32, bg);
            font.render(&entry.1, font_height as f32).draw(display, x + padding, y + padding, fg);

            y += entry_height + margin;
        }
    }

    fn key(mut self: Box<Self>, key: Key) -> Result<Option<Box<Screen>>> {
        match key {
            Key::Up if self.row > 0 => self.row -= 1,
            Key::Down if self.row + 1 < self.entries.len() => self.row += 1,
            Key::Escape => return MainScreen::new(1).map(Some),
            _ => (),
        }

        Ok(Some(self))
    }
}
