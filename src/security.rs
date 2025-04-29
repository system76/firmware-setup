use core::cell::Cell;
use core::cmp;
use core::ptr;

use ectool::{AccessLpcDirect, Ec, SecurityState, Timeout};
use orbclient::{Color, Renderer};
use std::prelude::*;
use std::proto::Protocol;
use std::uefi::{boot::InterfaceType, reset::ResetType};

use crate::display::{Display, Output};
use crate::key::{Key, key};
use crate::rng::Rng;
use crate::ui::Ui;

pub struct UefiTimeout {
    duration: u64,
    elapsed: Cell<u64>,
}

impl UefiTimeout {
    pub fn new(duration: u64) -> Self {
        Self {
            duration,
            elapsed: Cell::new(0),
        }
    }
}

impl Timeout for UefiTimeout {
    fn reset(&mut self) {
        self.elapsed.set(0);
    }

    fn running(&self) -> bool {
        let elapsed = self.elapsed.get() + 1;
        let _ = (std::system_table().BootServices.Stall)(1);
        self.elapsed.set(elapsed);
        elapsed < self.duration
    }
}

pub(crate) fn confirm(display: &mut Display) -> Result<()> {
    let (display_w, display_h) = (display.width(), display.height());

    let scale: i32 = if display_h > 1440 {
        4
    } else if display_h > 720 {
        2
    } else {
        1
    };

    // Style {
    let margin_lr = 12 * scale;
    let margin_tb = 4 * scale;

    let form_width = cmp::min(640 * scale as u32, display_w - margin_lr as u32 * 2);
    let form_x = (display_w as i32 - form_width as i32) / 2;

    let title_font_size = (12 * scale) as f32;
    let font_size = (10 * scale) as f32;
    // } Style

    let ui = Ui::new()?;
    let rng = Rng::one()?;

    // Clear any previous keys
    let _ = key(false);

    let title = "Firmware Update";
    let title_text = ui.font.render(title, title_font_size);

    let prompt = concat!(
        "Type in the following code to commence firmware flashing. The random code is a security ",
        "measure to ensure you have physical access to your device.",
    );
    let mut texts = ui.render_text_wrapped(prompt, font_size, form_width);

    // Add empty line
    texts.push(ui.font.render("", font_size));

    // Add code
    let mut code_bytes = [0; 4];
    rng.read(&mut code_bytes)?;
    let code = format!(
        "{:02}{:02}{:02}{:02}",
        code_bytes[0] % 100,
        code_bytes[1] % 100,
        code_bytes[2] % 100,
        code_bytes[3] % 100,
    );
    texts.push(ui.font.render(&code, font_size));

    let mut button_i = 0;
    let buttons = [
        ui.font.render("Confirm", font_size),
        ui.font.render("Cancel", font_size),
    ];

    let mut max_input = String::new();
    while max_input.len() < code.len() {
        // 0 is the widest number with Fira Sans
        max_input.push('0');
    }
    let max_input_text = ui.font.render(&max_input, font_size);

    let mut input = String::new();

    let help = concat!(
        "Cancel if you did not initiate the firmware flashing process. Firmware will not be ",
        "updated when canceled. The system will reboot to lock and secure the firmware.",
    );
    let help_texts = ui.render_text_wrapped(help, font_size, form_width);

    loop {
        let x = form_x;
        let mut y = margin_tb;

        display.set(ui.background_color);

        // Draw header
        {
            // TODO: Do not render in drawing loop
            let title_x = (display_w as i32 - title_text.width() as i32) / 2;
            title_text.draw(display, title_x, y, ui.text_color);
            y += title_font_size as i32 + margin_tb;

            display.rect(
                x - margin_lr / 2,
                y,
                form_width + margin_lr as u32,
                1,
                Color::rgb(0xac, 0xac, 0xac),
            );
            y += margin_tb * 2;
        }

        // Draw prompt and code
        for text in texts.iter() {
            text.draw(display, x, y, ui.text_color);
            y += font_size as i32;
        }
        y += margin_tb;

        // Draw input box
        let input_text = ui.font.render(&input, font_size);
        ui.draw_pretty_box(
            display,
            x,
            y,
            max_input_text.width(),
            font_size as u32,
            false,
        );
        input_text.draw(display, x, y, ui.text_color);
        if input.len() < code.len() {
            display.rect(
                x + input_text.width() as i32,
                y,
                font_size as u32 / 2,
                font_size as u32,
                ui.text_color,
            );
        }
        y += font_size as i32 + margin_tb;

        // Blank space
        y += font_size as i32;

        for (i, button_text) in buttons.iter().enumerate() {
            ui.draw_text_box(display, x, y, button_text, i == button_i, i == button_i);
            y += font_size as i32 + margin_tb;
        }

        // Draw footer
        {
            let mut bottom_y = display_h as i32;

            bottom_y -= margin_tb;
            for help in help_texts.iter().rev() {
                bottom_y -= font_size as i32;
                help.draw(display, x, bottom_y, ui.text_color);
            }

            bottom_y -= margin_tb * 3 / 2;
            display.rect(
                x - margin_lr / 2,
                bottom_y,
                form_width + margin_lr as u32,
                1,
                Color::rgb(0xac, 0xac, 0xac),
            );
        }

        display.sync();

        let k = key(true)?;
        match k {
            Key::Backspace => {
                input.pop();
            }
            Key::Character(c) => match c {
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    if input.len() < code.len() {
                        input.push(c);
                    }
                }
                _ => (),
            },
            Key::Enter => {
                if button_i == 0 {
                    if input == code {
                        // Continue if code entered
                        return Ok(());
                    } else {
                        // Clear invalid input
                        input.clear();
                    }
                } else {
                    // Return error if cancel selected
                    return Err(Status::ABORTED);
                }
            }
            Key::Escape => {
                input.clear();
            }
            Key::Down => {
                if button_i + 1 < buttons.len() {
                    button_i += 1;
                }
            }
            Key::Up => {
                button_i = button_i.saturating_sub(1);
            }
            _ => {}
        }
    }
}

extern "efiapi" fn run() -> bool {
    let access = match unsafe { AccessLpcDirect::new(UefiTimeout::new(100_000)) } {
        Ok(ok) => ok,
        Err(_) => return false,
    };

    let mut ec = match unsafe { Ec::new(access) } {
        Ok(ok) => ok,
        Err(_) => return false,
    };

    let security_state = match unsafe { ec.security_get() } {
        Ok(ok) => ok,
        Err(_) => return false,
    };

    // The EC will already be set to unlocked at this point, so the prompt
    // must be run even when in the "Unlock" state. This is fine, as the
    // prompt is for physical presence detection.

    if security_state == SecurityState::Lock {
        // Already locked, so do not confirm
        return false;
    }

    // Not locked, require confirmation

    let res = match Output::one() {
        Ok(output) => {
            let mut display = Display::new(output);

            let res = confirm(&mut display);

            // Clear display
            display.set(Color::rgb(0, 0, 0));
            display.sync();

            res
        }
        Err(err) => Err(err),
    };

    match res {
        Ok(()) => (),
        Err(_) => {
            // Lock on next shutdown, will power on automatically
            let _ = unsafe { ec.security_set(SecurityState::PrepareLock) };

            // Shutdown
            (std::system_table().RuntimeServices.ResetSystem)(
                ResetType::Shutdown,
                Status(0),
                0,
                ptr::null(),
            );
        }
    }

    true
}

pub const SYSTEM76_SECURITY_PROTOCOL_GUID: Guid = guid!("764247c4-a859-4a6b-b500-ed5d7a707dd4");
pub struct System76SecurityProtocol {
    #[allow(dead_code)]
    pub Run: extern "efiapi" fn() -> bool,
}

pub fn install() -> Result<()> {
    let uefi = std::system_table();

    //let uefi = unsafe { std::system_table_mut() };

    let protocol = Box::new(System76SecurityProtocol { Run: run });
    let protocol_ptr = Box::into_raw(protocol);
    let mut handle = Handle(0);
    Result::from((uefi.BootServices.InstallProtocolInterface)(
        &mut handle,
        &SYSTEM76_SECURITY_PROTOCOL_GUID,
        InterfaceType::Native,
        protocol_ptr as usize,
    ))?;

    Ok(())
}
