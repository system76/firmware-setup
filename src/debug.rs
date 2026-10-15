// SPDX-License-Identifier: GPL-3.0-only

use core::{
    fmt::{self, Write},
    ptr::{self, NonNull},
};
use uart_16550::{Config, Uart16550};

use crate::coreboot::COREBOOT_SERIAL;

#[allow(dead_code)]
pub struct Debug;

impl Write for Debug {
    fn write_str(&mut self, string: &str) -> Result<(), fmt::Error> {
        if let Some(ref serial) = *COREBOOT_SERIAL.lock() {
            match (serial.kind, serial.regwidth) {
                (1, 1) => {
                    let mut uart = unsafe { Uart16550::new_port(serial.baseaddr as u16).unwrap() };
                    uart.init(Config::default()).unwrap();
                    uart.send_bytes_exact(string.as_bytes());
                }
                (2, 4) => {
                    let addr = ptr::with_exposed_provenance_mut::<u8>(serial.baseaddr as usize);
                    let addr = NonNull::new(addr).unwrap();
                    let mut uart = unsafe { Uart16550::new_mmio(addr, 4).unwrap() };
                    uart.init(Config::default()).unwrap();
                    uart.send_bytes_exact(string.as_bytes());
                }
                _ => (),
            }
        } else {
            let mut uart = unsafe { Uart16550::new_port(0x402).unwrap() };
            uart.init(Config::default()).unwrap();
            uart.send_bytes_exact(string.as_bytes());
        }

        Ok(())
    }
}

#[cfg(not(feature = "debug"))]
pub fn _debug(_args: fmt::Arguments) {}

#[cfg(feature = "debug")]
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
