use alloc::boxed::Box;
use uefi::status::Result;

use display::Display;
use key::Key;

pub use self::main::MainScreen;
mod main;

pub use self::setting::SettingScreen;
mod setting;

pub trait Screen {
    fn draw(&self, display: &mut Display);
    fn key(self: Box<Self>, key: Key) -> Result<Option<Box<Screen>>>;
}
