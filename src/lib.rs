#![no_std]
#![feature(alloc)]
#![feature(asm)]
#![feature(compiler_builtins_lib)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(try_trait)]

#[macro_use]
extern crate alloc;
extern crate coreboot_table;
extern crate dmi;
extern crate ecflash;
extern crate orbclient;
extern crate plain;
extern crate uefi;
extern crate uefi_alloc;

use core::ops::Try;
use core::ptr;
use uefi::reset::ResetType;
use uefi::status::{Result, Status};

#[global_allocator]
static ALLOCATOR: uefi_alloc::Allocator = uefi_alloc::Allocator;

pub static mut HANDLE: uefi::Handle = uefi::Handle(0);
pub static mut UEFI: *mut uefi::system::SystemTable = 0 as *mut uefi::system::SystemTable;

#[macro_use]
mod macros;

pub mod app;
pub mod display;
pub mod exec;
pub mod fs;
pub mod hw;
pub mod image;
pub mod io;
pub mod loaded_image;
pub mod null;
pub mod panic;
pub mod pointer;
pub mod proto;
pub mod rt;
pub mod shell;
pub mod string;
pub mod text;
pub mod vars;

fn set_max_mode(output: &mut uefi::text::TextOutput) -> Result<()> {
    let mut max_i = None;
    let mut max_w = 0;
    let mut max_h = 0;

    for i in 0..output.Mode.MaxMode as usize {
        let mut w = 0;
        let mut h = 0;
        if (output.QueryMode)(output, i, &mut w, &mut h).into_result().is_ok() {
            if w >= max_w && h >= max_h {
                max_i = Some(i);
                max_w = w;
                max_h = h;
            }
        }
    }

    if let Some(i) = max_i {
        (output.SetMode)(output, i)?;
    }

    Ok(())
}

fn main() {
    let uefi = unsafe { &mut *::UEFI };

    let _ = (uefi.BootServices.SetWatchdogTimer)(0, 0, 0, ptr::null());

    /*
    if let Err(err) = set_max_mode(uefi.ConsoleOut).into_result() {
        println!("Failed to set max mode: {:?}", err);
    }
    */

    let _ = (uefi.ConsoleOut.SetAttribute)(uefi.ConsoleOut, 0x0F);

    if let Err(err) = app::main() {
        println!("App error: {:?}", err);
        let _ = io::wait_key();
    }

    unsafe {
        ((&mut *::UEFI).RuntimeServices.ResetSystem)(ResetType::Cold, Status(0), 0, ptr::null());
    }
}
