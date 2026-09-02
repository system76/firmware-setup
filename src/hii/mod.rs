// SPDX-License-Identifier: GPL-3.0-only

pub mod font;
pub mod string;

use std::prelude::*;
use std::proto::Protocol;
use std::uefi::hii::database::HiiDatabase;

#[allow(dead_code)]
pub struct Database(pub &'static mut HiiDatabase);

impl Protocol<HiiDatabase> for Database {
    fn guid() -> Guid {
        HiiDatabase::GUID
    }

    fn new(inner: &'static mut HiiDatabase) -> Self {
        Database(inner)
    }
}
