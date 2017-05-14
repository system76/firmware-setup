use core::fmt::{Write, Result};
use display::Display;
use orbclient::{Color, Renderer};

pub struct Console {
    x: i32,
    y: i32,
    bg: Color,
    fg: Color,
    display: Display,
}

impl Console {
    pub fn new(display: Display) -> Console {
        let mut console = Console {
            x: 0,
            y: 0,
            bg: Color::rgb(0, 0, 0),
            fg: Color::rgb(255, 255, 255),
            display: display,
        };

        console.display.set(console.bg);

        console
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.chars() {
            if c == '\n' {
                self.x = 0;
                self.y += 16;
            } else {
                self.display.rect(self.x, self.y, 8, 16, self.bg);
                self.display.char(self.x, self.y, c, self.fg);
                self.x += 8;
            }

            if self.x + 8 > self.display.width() as i32 {
                self.x = 0;
                self.y += 16;
            }

            while self.y + 16 > self.display.height() as i32 {
                self.display.scroll(16, self.bg);
                self.y -= 16;
            }
        }

        self.display.sync();

        Ok(())
    }
}