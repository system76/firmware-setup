use coreboot_table::{Mapper, PhysicalAddress, Serial, Table, VirtualAddress};
use hwio::{Mmio, Pio, Io};
use spin::Mutex;

use crate::serial::SerialPort;

pub struct PhysicalMapper;

impl Mapper for PhysicalMapper {
    unsafe fn map_aligned(&mut self, address: PhysicalAddress, size: usize) -> Result<VirtualAddress, &'static str> {
        Ok(VirtualAddress(address.0))
    }

    unsafe fn unmap_aligned(&mut self, address: VirtualAddress) -> Result<(), &'static str> {
        Ok(())
    }

    fn page_size(&self) -> usize {
        4096
    }
}

pub static COREBOOT_SERIAL: Mutex<Option<Serial>> = Mutex::new(None);

pub fn init() {
    let _ = coreboot_table::tables(|table| {
        match table {
            Table::Serial(serial) => {
                *COREBOOT_SERIAL.lock() = Some(serial.clone());
            },
            _ => (),
        }
        Ok(())
    }, &mut PhysicalMapper);
}
