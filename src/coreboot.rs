// SPDX-License-Identifier: GPL-3.0-only

use coreboot_table::{Mapper, PhysicalAddress, Serial, Table, VirtualAddress};
use spin::Mutex;

pub struct PhysicalMapper;

impl Mapper for PhysicalMapper {
    unsafe fn map_aligned(&mut self, address: PhysicalAddress, _size: usize) -> Result<VirtualAddress, &'static str> {
        Ok(VirtualAddress(address.0))
    }

    unsafe fn unmap_aligned(&mut self, _address: VirtualAddress) -> Result<(), &'static str> {
        Ok(())
    }

    fn page_size(&self) -> usize {
        4096
    }
}

pub static COREBOOT_SERIAL: Mutex<Option<Serial>> = Mutex::new(None);

pub fn init() {
    let _ = coreboot_table::tables(|table| {
        #[allow(clippy::single_match)]
        match table {
            Table::Serial(serial) => {
                *COREBOOT_SERIAL.lock() = Some(serial.clone());
            },
            _ => (),
        }
        Ok(())
    }, &mut PhysicalMapper);
}
