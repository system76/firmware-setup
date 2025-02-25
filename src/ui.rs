use core::ptr;

use orbclient::{Color, Renderer};
use orbfont::{Font, Text};

use std::prelude::*;

use crate::display::Display;
use crate::image::{self, Image};

static FONT_TTF: &[u8] = include_bytes!("../res/open-sans/OpenSans-Regular.ttf");
static CHECKBOX_CHECKED_BMP: &[u8] = include_bytes!("../res/checkbox_checked.bmp");
static CHECKBOX_UNCHECKED_BMP: &[u8] = include_bytes!("../res/checkbox_unchecked.bmp");

static mut FONT: *const Font = ptr::null_mut();
static mut CHECKBOX_CHECKED: *const Image = ptr::null_mut();
static mut CHECKBOX_UNCHECKED: *const Image = ptr::null_mut();

pub struct Ui {
    pub background_color: Color,
    pub highlight_color: Color,
    pub outline_color: Color,
    pub text_color: Color,
    pub highlight_text_color: Color,
    pub font: &'static Font,
    checkbox_checked: &'static Image,
    checkbox_unchecked: &'static Image,
}

impl Ui {
    pub fn new() -> Result<Self> {
        let background_color = Color::rgb(0x1B, 0x1B, 0x1B);
        let highlight_color = Color::rgb(0x63, 0xD0, 0xDF);
        let outline_color = Color::rgba(0xfe, 0xff, 0xff, 0xc4);
        let text_color = Color::rgb(0xC4, 0xC4, 0xC4);
        let highlight_text_color = Color::rgb(0x00, 0x00, 0x00);

        let font = unsafe {
            if FONT.is_null() {
                let font = match Font::from_data(FONT_TTF) {
                    Ok(ok) => ok,
                    Err(err) => {
                        println!("failed to parse font: {}", err);
                        return Err(Status::NOT_FOUND);
                    }
                };
                FONT = Box::into_raw(Box::new(font));
            }
            &*FONT
        };

        let checkbox_checked = unsafe {
            if CHECKBOX_CHECKED.is_null() {
                let image = match image::bmp::parse(CHECKBOX_CHECKED_BMP) {
                    Ok(ok) => ok,
                    Err(err) => {
                        println!("failed to parse checkbox checked: {}", err);
                        return Err(Status::NOT_FOUND);
                    }
                };
                CHECKBOX_CHECKED = Box::into_raw(Box::new(image));
            }
            &*CHECKBOX_CHECKED
        };

        let checkbox_unchecked = unsafe {
            if CHECKBOX_UNCHECKED.is_null() {
                let image = match image::bmp::parse(CHECKBOX_UNCHECKED_BMP) {
                    Ok(ok) => ok,
                    Err(err) => {
                        println!("failed to parse checkbox unchecked: {}", err);
                        return Err(Status::NOT_FOUND);
                    }
                };
                CHECKBOX_UNCHECKED = Box::into_raw(Box::new(image));
            }
            &*CHECKBOX_UNCHECKED
        };

        Ok(Self {
            background_color,
            highlight_color,
            outline_color,
            text_color,
            highlight_text_color,
            font,
            checkbox_checked,
            checkbox_unchecked,
        })
    }

    //TODO: move to orbfont and optimize
    pub fn render_text_wrapped(&self, string: &str, font_size: f32, width: u32) -> Vec<Text> {
        let mut texts = Vec::new();

        //TODO: support different whitespace differently, like newline?
        let words: Vec<&str> = string.split_whitespace().collect();

        let mut line = String::new();
        let mut last_text_opt = None;
        let mut i = 0;
        while i < words.len() {
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(words[i]);

            let text = self.font.render(&line, font_size);
            if text.width() > width {
                line.clear();
                if let Some(last_text) = last_text_opt.take() {
                    texts.push(last_text);
                    // Process this word again
                    continue;
                } else {
                    texts.push(text);
                }
            } else {
                last_text_opt = Some(text);
            }

            i += 1;
        }

        if let Some(last_text) = last_text_opt.take() {
            texts.push(last_text);
        }

        texts
    }

    pub fn draw_pretty_box(
        &self,
        display: &mut Display,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        highlighted: bool,
    ) {
        let (_display_w, display_h) = (display.width(), display.height());

        let scale = if display_h > 1440 {
            4
        } else if display_h > 720 {
            2
        } else {
            1
        };

        // Style {
        let padding_lr = 4 * scale;
        let padding_tb = 2 * scale;
        // } Style

        display.rounded_rect(
            x - padding_lr,
            y - padding_tb,
            w + (padding_lr * 2) as u32,
            h + (padding_tb * 2) as u32,
            0,
            highlighted,
            if highlighted {
                self.highlight_color
            } else {
                self.outline_color
            }
        );
    }

    pub fn draw_text_box(
        &self,
        display: &mut Display,
        x: i32,
        y: i32,
        rendered: &Text,
        pretty_box: bool,
        highlighted: bool,
    ) {
        if pretty_box {
            self.draw_pretty_box(
                display,
                x,
                y,
                rendered.width(),
                rendered.height(),
                highlighted,
            );
        }
        let text_color = if highlighted {
            self.highlight_text_color
        } else {
            self.text_color
        };
        rendered.draw(display, x, y, text_color);
    }

    pub fn draw_check_box(&self, display: &mut Display, x: i32, y: i32, value: bool) -> i32 {
        let checkbox = if value {
            self.checkbox_checked
        } else {
            self.checkbox_unchecked
        };
        checkbox.draw(display, x, y);
        checkbox.height() as i32
    }
}
