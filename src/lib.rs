#![no_std]
#![feature(asm)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(prelude_import)]
#![feature(try_trait)]

extern crate coreboot_table;
extern crate dmi;
extern crate ecflash;
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

mod app;
mod debug;
mod display;
mod hii;
pub mod image;
mod io;
mod key;
pub mod null;
pub mod text;

fn set_max_mode(output: &uefi::text::TextOutput) -> Result<()> {
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

// HII {

use hwio::{Io, Pio};
use std::mem;
use uefi::hii::database::HiiHandle;
use uefi::hii::ifr::IfrOpHeader;
use uefi::hii::package::{HiiPackageHeader, HiiPackageKind, HiiPackageListHeader};
use uefi::reset::ResetType;
use uefi::status::Error;

fn dump_forms(data: &[u8]) {
    let mut i = 0;
    while i + mem::size_of::<IfrOpHeader>() < data.len() {
        let form = unsafe {
            & *(data.as_ptr().add(i) as *const IfrOpHeader)
        };

        debugln!(
            "      Form: OpCode {:#x} {:?}, Length {}, Scope {}",
            form.OpCode as u8,
            form.OpCode,
            form.Length(),
            form.Scope()
        );

        i += form.Length() as usize;
    }
}

fn dump_package(package: &HiiPackageHeader) {
    debugln!(
        "    Package: Kind {:#x} {:?}, Length {}",
        package.Kind() as u8,
        package.Kind(),
        package.Length()
    );

    let data = package.Data();
    match package.Kind() {
        HiiPackageKind::Forms => dump_forms(data),
        _ => (),
    }
}

fn dump_package_list(package_list: &HiiPackageListHeader) {
    debugln!(
        "  Package List: Guid {}, Length {}",
        package_list.PackageListGuid,
        package_list.PackageLength
    );

    let data = package_list.Data();
    let mut i = 0;
    while i + mem::size_of::<HiiPackageHeader>() < data.len() {
        let package = unsafe {
            & *(data.as_ptr().add(i) as *const HiiPackageHeader)
        };
        dump_package(package);
        i += package.Length() as usize;
    }
}

fn dump_package_lists(data: &[u8]) {
    debugln!("Package Lists: {}", data.len());

    let mut i = 0;
    while i + mem::size_of::<HiiPackageListHeader>() < data.len() {
        let package_list = unsafe {
            & *(data.as_ptr().add(i) as *const HiiPackageListHeader)
        };
        dump_package_list(package_list);
        i += package_list.PackageLength as usize;
    }
}

fn dump_hii() -> Result<()> {
    for db in hii::Database::all() {
        let mut size = 0;

        match (db.0.ExportPackageLists)(
            db.0,
            HiiHandle(0),
            &mut size,
            unsafe { &mut *ptr::null_mut() }
        ).into_result() {
            Ok(_) => (),
            Err(err) if err == Error::BufferTooSmall => (),
            Err(err) => return Err(err),
        }

        let mut data: Box<[u8]> = vec![0; size].into_boxed_slice();
        (db.0.ExportPackageLists)(
            db.0,
            HiiHandle(0),
            &mut size,
            unsafe { &mut *(data.as_mut_ptr() as *mut HiiPackageListHeader) }
        )?;

        if size != data.len() {
            debugln!("Database: ExportPackageLists size {} does not match {}", size, data.len());
            return Err(Error::BadBufferSize);
        }

        dump_package_lists(&data);
    }

    debugln!("Shutdown");
    Pio::<u16>::new(0x604).write(0x2000);

    Ok(())
}

// } HII

#[no_mangle]
pub extern "C" fn main() -> Status {
    let uefi = std::system_table();

    let _ = (uefi.BootServices.SetWatchdogTimer)(0, 0, 0, ptr::null());

    if let Err(err) = set_max_mode(uefi.ConsoleOut).into_result() {
        println!("Failed to set max mode: {:?}", err);
    }

    let _ = (uefi.ConsoleOut.SetAttribute)(uefi.ConsoleOut, 0x0F);

    if let Err(err) = /*app::main()*/ dump_hii() {
        println!("App error: {:?}", err);
        let _ = io::wait_key();
    }

    Status(0)
}
