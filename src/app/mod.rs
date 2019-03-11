use core::char;
use orbclient::{Color, Renderer};
use uefi::status::{Error, Result, Status};
use uefi::text::TextInputKey;

use display::{Display, Output};
use fs::load;
use image::{self, Image};
use proto::Protocol;
use text::TextDisplay;

static SPLASHBMP: &'static str = concat!("\\", env!("BASEDIR"), "\\res\\splash.bmp");

#[derive(Debug)]
enum Key {
    Backspace,
    Tab,
    Enter,
    Character(char),
    Up,
    Down,
    Right,
    Left,
    Home,
    End,
    Insert,
    Delete,
    PageUp,
    PageDown,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Escape,
    Scancode(u16),
}

fn key() -> Result<Key> {
    let uefi = unsafe { &mut *::UEFI };

    let mut index = 0;
    (uefi.BootServices.WaitForEvent)(1, &uefi.ConsoleIn.WaitForKey, &mut index)?;

    let mut input = TextInputKey {
        ScanCode: 0,
        UnicodeChar: 0
    };

    (uefi.ConsoleIn.ReadKeyStroke)(uefi.ConsoleIn, &mut input)?;

    Ok(match input.ScanCode {
        0 => match unsafe { char::from_u32_unchecked(input.UnicodeChar as u32) } {
            '\u{8}' => Key::Backspace,
            '\t' => Key::Tab,
            '\r' => Key::Enter,
            c => Key::Character(c),
        },
        1 => Key::Up,
        2 => Key::Down,
        3 => Key::Right,
        4 => Key::Left,
        5 => Key::Home,
        6 => Key::End,
        7 => Key::Insert,
        8 => Key::Delete,
        9 => Key::PageUp,
        10 => Key::PageDown,
        11 => Key::F1,
        12 => Key::F2,
        13 => Key::F3,
        14 => Key::F4,
        15 => Key::F5,
        16 => Key::F6,
        17 => Key::F7,
        18 => Key::F8,
        19 => Key::F9,
        20 => Key::F10,
        21 => Key::F11,
        22 => Key::F12,
        23 => Key::Escape,
        scancode => Key::Scancode(scancode),
    })
}

fn inner() -> Result<()> {
    loop {
        println!("{:?}", key());
    }
}

pub fn main() -> Result<()> {
    let uefi = unsafe { &mut *::UEFI };

    let mut display = {
        let output = Output::one()?;

        /*
        let mut max_i = 0;
        let mut max_w = 0;
        let mut max_h = 0;

        for i in 0..output.0.Mode.MaxMode {
            let mut mode_ptr = ::core::ptr::null_mut();
            let mut mode_size = 0;
            (output.0.QueryMode)(output.0, i, &mut mode_size, &mut mode_ptr)?;

            let mode = unsafe { &mut *mode_ptr };
            let w = mode.HorizontalResolution;
            let h = mode.VerticalResolution;
            if w >= max_w && h >= max_h {
                max_i = i;
                max_w = w;
                max_h = h;
            }
        }

        let _ = (output.0.SetMode)(output.0, max_i);
        */

        Display::new(output)
    };

    let mut splash = Image::new(0, 0);
    {
        println!("Loading Splash...");
        if let Ok(data) = load(SPLASHBMP) {
            if let Ok(image) = image::bmp::parse(&data) {
                splash = image;
            }
        }
        println!(" Done");
    }

    {
        let bg = Color::rgb(0x41, 0x3e, 0x3c);

        display.set(bg);

        {
            let x = (display.width() as i32 - splash.width() as i32)/2;
            let y = 16;
            splash.draw(&mut display, x, y);
        }

        {
            let prompt = concat!("Firmware Setup ", env!("CARGO_PKG_VERSION"));
            let mut x = (display.width() as i32 - prompt.len() as i32 * 8)/2;
            let y = display.height() as i32 - 64;
            for c in prompt.chars() {
                display.char(x, y, c, Color::rgb(0xff, 0xff, 0xff));
                x += 8;
            }
        }

        display.sync();
    }

    {
        let cols = 80;
        let off_x = (display.width() as i32 - cols as i32 * 8)/2;
        let off_y = 16 + splash.height() as i32 + 16;
        let rows = (display.height() as i32 - 64 - off_y - 1) as usize/16;
        display.rect(off_x, off_y, cols as u32 * 8, rows as u32 * 16, Color::rgb(0, 0, 0));
        display.sync();

        let mut text = TextDisplay::new(&mut display);
        text.off_x = off_x;
        text.off_y = off_y;
        text.cols = cols;
        text.rows = rows;
        text.pipe(inner)?;
    }

    Ok(())
}
