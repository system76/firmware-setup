use core::str;
use coreboot_table::{Mapper, PhysicalAddress, VirtualAddress, Table, CmosRecord};
use std::collections::BTreeMap;

pub struct IdentityMapper;

impl Mapper for IdentityMapper {
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

pub fn coreboot() -> Result<(Vec<(String, u32)>, BTreeMap<u32, Vec<(String, u32)>>), &'static str> {
    let mut entries = Vec::new();
    let mut enums_map = BTreeMap::new();

    coreboot_table::tables(|table| {
        match table {
            Table::Cmos(cmos) => {
                println!("{:?}", cmos);
                for record in cmos.records() {
                    match record {
                        CmosRecord::Entry(entry) => {
                            let name = str::from_utf8(entry.name()).unwrap();
                            println!("    {}: {:?}", name, entry);
                            entries.push(
                                (name.to_string(), entry.config_id)
                            );
                        },
                        CmosRecord::Enum(enum_) => {
                            let text = str::from_utf8(enum_.text()).unwrap();
                            println!("    {}: {:?}", text, enum_);
                            (*enums_map.entry(enum_.config_id).or_insert(Vec::new())).push(
                                (text.to_string(), enum_.value)
                            );
                        },
                        CmosRecord::Other(other) => {
                            println!("    {:?}", other);
                        },
                    }
                }
            },
            Table::Framebuffer(framebuffer) => {
                println!("{:?}", framebuffer);
            },
            Table::Memory(memory) => {
                println!("{:?}", memory);
                for range in memory.ranges() {
                    println!("    {:?}", range);
                }
            },
            Table::Other(other) => {
                println!("{:?}", other);
            },
        }
        Ok(())
    }, &mut IdentityMapper)?;

    Ok((entries, enums_map))
}
