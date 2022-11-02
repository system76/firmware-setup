// SPDX-License-Identifier: GPL-3.0-only

use hwio::{Io, Pio};
use std::{char, mem, ptr, str};
use std::ops::Try;
use std::proto::Protocol;
use std::uefi::hii::database::HiiHandle;
use std::uefi::hii::ifr::{IfrOpCode, IfrOpHeader, IfrForm, IfrAction};
use std::uefi::hii::package::{HiiPackageHeader, HiiPackageKind, HiiPackageListHeader, HiiStringPackageHeader};
use std::uefi::hii::sibt::{SibtHeader, SibtKind, SibtEnd, SibtSkip2, SibtStringUcs2, SibtStringsUcs2};
use std::uefi::status::{Error, Result};

use crate::hii;

// The IfrOpCode's we need to handle so far:
// Action
// DefaultStore
// End
// Form
// FormSet
// Guid
// OneOf
// OneOfOption
// Ref
// Subtitle

fn dump_form(op: &IfrOpHeader) {
    debugln!(
        "    Form: OpCode {:#x} {:?}, Length {}, Scope {}",
        op.OpCode as u8,
        op.OpCode,
        op.Length(),
        op.Scope()
    );

    match op.OpCode {
        IfrOpCode::Form => {
            let form = unsafe { &*(op as *const _ as *const IfrForm) };
            debugln!("      {:?}", form);
        },
        IfrOpCode::Action => {
            let action = unsafe { &*(op as *const _ as *const IfrAction) };
            debugln!("      {:?}", action);
        },
        _ => ()
    }
}

fn dump_forms(data: &[u8]) {
    let mut i = 0;
    while i + mem::size_of::<IfrOpHeader>() < data.len() {
        let op = unsafe {
            & *(data.as_ptr().add(i) as *const IfrOpHeader)
        };

        dump_form(op);

        i += op.Length() as usize;
    }
}

fn dump_strings(strings: &HiiStringPackageHeader) {
    debugln!("    {:?}: {:?}", strings, str::from_utf8(strings.Language()));
    let info = strings.StringInfo();
    let mut id = 1;
    let mut i = 0;
    while i < info.len() {
        let ptr = unsafe { info.as_ptr().add(i) };
        let header = unsafe { &*(ptr as *const SibtHeader) };
        match header.BlockType {
            SibtKind::End => {
                let block = unsafe { &*(ptr as *const SibtEnd) };
                debugln!("      {:?}", block);
                i += mem::size_of_val(block);
            },
            SibtKind::Skip2 => {
                let block = unsafe { &*(ptr as *const SibtSkip2) };
                debugln!("      {:?}", block);
                id += block.SkipCount();
                i += mem::size_of_val(block);
            },
            SibtKind::StringUcs2 => {
                let block = unsafe { &*(ptr as *const SibtStringUcs2) };
                debugln!("      {:?}", block);
                let text = block.StringText();
                {
                    // Capacity will be at least text.len()
                    let mut string = String::with_capacity(text.len());
                    for &w in text.iter() {
                        let c = unsafe { char::from_u32_unchecked(w as u32) };
                        string.push(c);
                    }
                    debugln!("        {}: \"{}\"", id, string);
                }
                id += 1;
                let text_end = unsafe { text.as_ptr().add(text.len() + 1) };
                i += text_end as usize - ptr as usize
            },
            /* TODO, if required
            SibtKind::StringsUcs2 => {
                let block = unsafe { &*(ptr as *const SibtStringsUcs2) };
                debugln!("      {:?}", block);
                if block.StringCount > 0 {
                    let text = block.StringText(block.StringCount - 1);
                    let text_end = unsafe { text.as_ptr().add(text.len() + 1) };
                    i += text_end as usize - ptr as usize
                } else {
                    i += mem::size_of_val(block);
                }
            },
            */
            unknown => {
                panic!(
                    "Unimplemented SibtKind {:?}, {:#x}",
                    unknown,
                    unknown as u8
                );
            }
        }
    }
}

fn dump_package(package: &HiiPackageHeader) {
    debugln!(
        "  Package: Kind {:#x} {:?}, Length {}",
        package.Kind() as u8,
        package.Kind(),
        package.Length()
    );

    match package.Kind() {
        HiiPackageKind::Forms => {
            dump_forms(package.Data());
        },
        HiiPackageKind::Strings => {
            let strings = unsafe { &*(package as *const _ as *const HiiStringPackageHeader) };
            dump_strings(strings);
        }
        _ => (),
    }
}

fn dump_package_list(package_list: &HiiPackageListHeader) {
    debugln!(
        "Package List: Guid {}, Length {}",
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
    let mut i = 0;
    while i + mem::size_of::<HiiPackageListHeader>() < data.len() {
        let package_list = unsafe {
            & *(data.as_ptr().add(i) as *const HiiPackageListHeader)
        };
        dump_package_list(package_list);
        i += package_list.PackageLength as usize;
    }
}

pub fn dump_hii() -> Result<()> {
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

    //debugln!("Shutdown");
    //Pio::<u16>::new(0x604).write(0x2000);

    Ok(())
}
