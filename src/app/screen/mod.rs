use alloc::boxed::Box;
use orbfont::Font;
use uefi::status::Result;

use display::Display;
use key::Key;

pub use self::boot::BootScreen;
mod boot;

pub use self::main::MainScreen;
mod main;

pub use self::setting::SettingScreen;
mod setting;

pub trait Screen {
    fn draw(&self, display: &mut Display, font: &Font);
    fn key(self: Box<Self>, key: Key) -> Result<Option<Box<Screen>>>;
}
