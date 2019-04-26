use core::fmt::{self, Write};
use hwio::{Io, Pio};

pub struct Debug;

impl Write for Debug {
    fn write_str(&mut self, string: &str) -> Result<(), fmt::Error> {
        let mut port = Pio::<u8>::new(0x402);
        for b in string.bytes() {
            port.write(b);
        }

        Ok(())
    }
}

pub fn _debug(args: fmt::Arguments) {
    Debug.write_fmt(args).unwrap();
}


#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ($crate::debug::_debug(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! debugln {
    () => (debug!("\n"));
    ($fmt:expr) => (debug!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (debug!(concat!($fmt, "\n"), $($arg)*));
}
