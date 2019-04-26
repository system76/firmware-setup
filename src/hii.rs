use std::proto::Protocol;
use uefi::hii::database::HiiDatabase;
use uefi::guid::{Guid, HII_DATABASE_GUID};

pub struct Database(pub &'static mut HiiDatabase);

impl Protocol<HiiDatabase> for Database {
    fn guid() -> Guid {
        HII_DATABASE_GUID
    }

    fn new(inner: &'static mut HiiDatabase) -> Self {
        Database(inner)
    }
}
