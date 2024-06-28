// SPDX-License-Identifier: GPL-3.0-only

#![no_std]
#![no_main]
#![allow(non_snake_case)]

#[macro_use]
extern crate memoffset;
#[macro_use]
extern crate uefi_std as std;

use std::prelude::*;

use core::ptr;

#[macro_use]
mod debug;

mod coreboot;
mod display;
mod hii;
pub mod image;
mod key;
mod rng;
mod serial;

//mod dump_hii;
mod fde;
mod security;
mod ui;

#[no_mangle]
pub extern "C" fn main() -> Status {
    let uefi = std::system_table();

    let _ = (uefi.BootServices.SetWatchdogTimer)(0, 0, 0, ptr::null());

    coreboot::init();

    if let Err(err) = fde::Fde::install() {
        println!("Fde error: {:?}", err);
        let _ = key::key(true);
    }

    if let Err(err) = security::install() {
        debugln!("security error: {:?}", err);
        let _ = key::key(true);
    }

    Status(0)
}
