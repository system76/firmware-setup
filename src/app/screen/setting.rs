use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::str;
use coreboot_table::{self, Table, CmosRecord};
use orbclient::{Color, Renderer};
use uefi::status::{Error, Result};

use display::Display;
use key::Key;

use super::{Screen, MainScreen};
use super::super::coreboot::IdentityMapper;

pub struct SettingScreen {
    entries: Vec<(String, u32)>,
    enums_map: BTreeMap<u32, Vec<(String, u32)>>,
    selected: usize,
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
                                let name = str::from_utf8(entry.name()).unwrap();
                                entries.push(
                                    (name.to_string(), entry.config_id)
                                );
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

        Ok(Box::new(SettingScreen {
            entries: entries,
            enums_map: enums_map,
            selected: 0,
        }))
    }
}

impl Screen for SettingScreen {
    fn draw(&self, display: &mut Display) {
        let gray = Color::rgb(0x41, 0x3e, 0x3c);
        let black = Color::rgb(0, 0, 0);
        let white = Color::rgb(0xFF, 0xFF, 0xFF);

        let mut y = (display.height() as i32 - self.entries.len() as i32 * 16)/2;

        for (i, entry) in self.entries.iter().enumerate() {
            let (fg, bg) = if i == self.selected {
                (black, white)
            } else {
                (white, gray)
            };

            let mut x = (display.width() as i32 - /* entry.0.len() as i32 * 8 */ 400)/2;
            for c in entry.0.chars() {
                display.rect(x, y, 8, 16, bg);
                display.char(x, y, c, fg);
                x += 8;
            }

            y += 16;

            if let Some(enums) = self.enums_map.get(&entry.1) {
                for (name, _value) in enums.iter() {
                    let mut x = (display.width() as i32 - /* entry.0.len() as i32 * 8 */ 400)/2 + 200;
                    for c in name.chars() {
                        display.rect(x, y, 8, 16, bg);
                        display.char(x, y, c, fg);
                        x += 8;
                    }

                    y += 16;
                }
            }
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
            Key::Escape => return MainScreen::new().map(Some),
            _ => (),
        }

        Ok(Some(self))
    }
}
