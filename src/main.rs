// SPDX-License-Identifier: GPL-3.0-only

#![no_std]
#![no_main]

//#[macro_use]
extern crate alloc;

//mod display;
//mod fde;
//mod image;
//mod key;
//mod security;
//mod ui;

mod edk2;
mod fde_new;

use uefi::prelude::*;

#[entry]
fn main() -> Status {
    //uefi::helpers::init().unwrap();

    let _ = boot::set_watchdog_timer(0, 0, None);

    let _ = fde_new::replace_interface();
    //let _ = security::install();

    Status::SUCCESS
}
