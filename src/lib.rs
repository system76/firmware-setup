#![no_std]
#![feature(asm)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(prelude_import)]
#![feature(try_trait)]
#![allow(non_snake_case)]

extern crate coreboot_table;
extern crate dmi;
extern crate ecflash;
#[macro_use]
extern crate memoffset;
extern crate orbclient;
extern crate orbfont;
extern crate plain;
#[macro_use]
extern crate uefi_std as std;

#[allow(unused_imports)]
#[prelude_import]
use std::prelude::*;

use core::ops::Try;
use core::ptr;
use std::proto::Protocol;
use uefi::status::{Result, Status};

#[macro_use]
mod debug;
mod display;
mod hii;
pub mod image;
mod key;
pub mod null;
pub mod text;

//mod dump_hii;
mod fde;

#[no_mangle]
pub extern "C" fn main() -> Status {
    let uefi = std::system_table();

    let _ = (uefi.BootServices.SetWatchdogTimer)(0, 0, 0, ptr::null());

    if let Err(err) = fde::Fde::install() {
        println!("Fde error: {:?}", err);
        let _ = key::key();
    }

    Status(0)
}
