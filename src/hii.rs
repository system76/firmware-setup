// SPDX-License-Identifier: GPL-3.0-only

use std::proto::Protocol;
use std::prelude::*;
use std::uefi::hii::database::HiiDatabase;
use std::uefi::guid::HII_DATABASE_GUID;

#[allow(dead_code)]
pub struct Database(pub &'static mut HiiDatabase);

impl Protocol<HiiDatabase> for Database {
    fn guid() -> Guid {
        HII_DATABASE_GUID
    }

    fn new(inner: &'static mut HiiDatabase) -> Self {
        Database(inner)
    }
}
