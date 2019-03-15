use orbclient::{Color, Renderer};
use orbfont::Font;
use uefi::status::{Error, Result};

use crate::display::{Display, Output};
use crate::fs::load;
use crate::image::{self, Image};
use crate::key::key;
use crate::proto::Protocol;

mod coreboot;

use self::screen::MainScreen;
mod screen;

static SPLASHBMP: &'static str = concat!("\\", env!("BASEDIR"), "\\res\\splash.bmp");
static FONTTTF: &'static str = concat!("\\", env!("BASEDIR"), "\\res\\FiraSans-Regular.ttf");

pub fn main() -> Result<()> {
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

    let font;
    {
        println!("Loading Font...");
        match Font::from_data(load(FONTTTF)?) {
            Ok(ok) => font = ok,
            Err(err) => {
                println!("failed to parse font: {}", err);
                return Err(Error::NotFound);
            }
        }
        println!(" Done");
    }

    let mut screen = MainScreen::new(0)?;
    loop {
        display.set(Color::rgb(0x41, 0x3e, 0x3c));

        {
            let x = (display.width() as i32 - splash.width() as i32)/2;
            let y = 16;
            splash.draw(&mut display, x, y);
        }

        {
            let prompt = concat!("Firmware Setup ", env!("CARGO_PKG_VERSION"));

            let text = font.render(prompt, 24.0);
            let x = (display.width() as i32 - text.width() as i32)/2;
            let y = display.height() as i32 - 64;
            text.draw(&mut display, x, y, Color::rgb(0xff, 0xff, 0xff));
        }

        screen.draw(&mut display, &font);

        display.sync();

        let key = key()?;
        screen = match screen.key(key)? {
            Some(some) => some,
            None => break
        };
    }

    Ok(())
}
