use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::{char, str};
use coreboot_table::{self, Table, CmosRecord};
use orbclient::{Color, Renderer};
use orbfont::Font;
use uefi::status::{Error, Result};

use crate::display::Display;
use crate::key::Key;

use super::{Screen, MainScreen};
use super::super::coreboot::IdentityMapper;

unsafe fn cmos_read(index: u8) -> u8 {
    let value;
    asm!(
        "
        cli
        out 0x70, al
        in al, 0x71
        sti
        "
        : "={al}"(value)
        : "{al}"(index)
        :
        : "volatile", "intel"
    );
    value
}

unsafe fn nvram_read(mut bit: u32, mut len: u32) -> u32 {
    let mut data = 0;

    let mut index = 0;
    let mut byte = 0;
    let mut mask = 1;
    while len > 0 {
        let new_index = (bit / 8) as u8;
        if mask == 1 || index != new_index {
            index = new_index;
            byte = cmos_read(index);
        }

        if byte & (1 << (bit % 8)) > 0 {
            data |= mask;
        }

        len -= 1;
        bit += 1;
        mask <<= 1;
    }

    data
}

pub struct SettingScreen {
    entries: Vec<(String, u32, u32, u32)>,
    enums_map: BTreeMap<u32, Vec<(String, u32)>>,
    row: usize,
    column: usize,
}

impl SettingScreen {
    pub fn new() -> Result<Box<Screen>> {
        let mut entries = Vec::new();
        let mut enums_map = BTreeMap::new();

        coreboot_table::tables(|table| {
            match table {
                Table::Cmos(cmos) => {
                    for record in cmos.records() {
                        match record {
                            CmosRecord::Entry(entry) => {
                                match unsafe { char::from_u32_unchecked(entry.config) } {
                                    'e' | 'h' if entry.length > 0 => {
                                        let name = str::from_utf8(entry.name()).unwrap();
                                        let value = unsafe { nvram_read(entry.bit, entry.length) };
                                        entries.push((
                                            name.to_string(),
                                            entry.config_id,
                                            value,
                                            (1 << entry.length) - 1
                                        ));
                                    },
                                    _ => ()
                                }
                            },
                            CmosRecord::Enum(enum_) => {
                                let text = str::from_utf8(enum_.text()).unwrap();
                                (*enums_map.entry(enum_.config_id).or_insert(Vec::new())).push(
                                    (text.to_string(), enum_.value)
                                );
                            },
                            _ => (),
                        }
                    }
                },
                _ => (),
            }
            Ok(())
        }, &mut IdentityMapper).map_err(|err| {
            println!("failed to parse coreboot tables: {}", err);
            Error::NotFound
        })?;

        //entries.retain(|entry| enums_map.contains_key(&entry.1));

        Ok(Box::new(SettingScreen {
            entries: entries,
            enums_map: enums_map,
            row: 0,
            column: 0,
        }))
    }
}

impl Screen for SettingScreen {
    fn draw(&self, display: &mut Display, font: &Font) {
        let font_height = 24;
        let padding = 16;
        let margin = 12;

        let entry_height = padding + font_height + padding;

        let mut y = (display.height() as i32 + 64 - self.entries.len() as i32 * (entry_height + margin))/2;

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

            if i == self.row && 0 == self.column {
                display.rounded_rect(x - 2, y - 2, entry_width as u32 + 4, entry_height as u32 + 4, 8, true, Color::rgb(0x94, 0x94, 0x94));
                display.rounded_rect(x + 2, y + 2, entry_width as u32 - 4, entry_height as u32 - 4, 6, true, bg);
            } else {
                display.rect(x, y, entry_width as u32, entry_height as u32, bg);
            }

            font.render(&entry.0, font_height as f32).draw(display, x + padding, y + padding, fg);

            x += entry_width + margin;

            if i == self.row && 1 == self.column {
                display.rounded_rect(x - 2, y - 2, entry_width as u32 + 4, entry_height as u32 + 4, 8, true, Color::rgb(0x94, 0x94, 0x94));
                display.rounded_rect(x + 2, y + 2, entry_width as u32 - 4, entry_height as u32 - 4, 6, true, bg);
            } else {
                display.rect(x, y, entry_width as u32, entry_height as u32, bg);
            }

            let mut selected_opt = None;
            if let Some(enums) = self.enums_map.get(&entry.1) {
                for (name, value) in enums.iter() {
                    if *value == entry.2 {
                        selected_opt = Some(name);
                        break;
                    }
                }
            }

            if let Some(selected) = selected_opt {
                font.render(&format!("{}: {}", entry.2, selected), font_height as f32).draw(display, x + padding, y + padding, fg);
            } else {
                font.render(&format!("{}", entry.2), font_height as f32).draw(display, x + padding, y + padding, fg);
            }

            y += entry_height + margin;
        }
    }

    fn key(mut self: Box<Self>, key: Key) -> Result<Option<Box<Screen>>> {
        match key {
            Key::Up if self.row > 0 => self.row -= 1,
            Key::Down if self.row + 1 < self.entries.len() => self.row += 1,
            Key::Left if self.column > 0 => self.column -= 1,
            Key::Right if self.column < 1 => self.column += 1,
            Key::Enter if self.column == 0 => self.column = 1,
            Key::Escape if self.column == 0 => return MainScreen::new(2).map(Some),
            Key::Escape if self.column > 0 => self.column -= 1,
            Key::PageUp => {
                let row = self.row;
                if let Some(entry) = self.entries.get_mut(row) {
                    if entry.2 > 0 {
                        entry.2 -= 1;
                    }
                }
            },
            Key::PageDown => {
                let row = self.row;
                if let Some(entry) = self.entries.get_mut(row) {
                    if entry.2 < entry.3 {
                        entry.2 += 1;
                    }
                }
            },
            _ => (),
        }

        Ok(Some(self))
    }
}
