use core::ptr;
use std::proto::Protocol;
use std::prelude::*;

pub const RNG_PROTOCOL_GUID: Guid = Guid(0x3152bca5, 0xeade, 0x433d, [0x86, 0x2e, 0xc0, 0x1c, 0xdc, 0x29, 0x1f, 0x44]);

pub struct Rng(pub &'static mut RngProtocol);

impl Rng {
    pub fn read(&self, buf: &mut [u8]) -> Result<()> {
        Result::from((self.0.GetRNG)(
            self.0,
            ptr::null(),
            buf.len(),
            buf.as_mut_ptr(),
        ))?;
        Ok(())
    }
}

impl Protocol<RngProtocol> for Rng {
    fn guid() -> Guid {
        RNG_PROTOCOL_GUID
    }

    fn new(inner: &'static mut RngProtocol) -> Self {
        Rng(inner)
    }
}

#[repr(C)]
pub struct RngProtocol {
    pub GetInfo: extern "efiapi" fn(
        &RngProtocol,
        RNGAlgorithmListSize: &mut usize,
        RNGAlgorithmList: *mut Guid,
    ) -> Status,
    pub GetRNG: extern "efiapi" fn(
        &RngProtocol,
        RNGAlgorithm: *const Guid,
        RNGValueLength: usize,
        RNGValue: *mut u8,
    ) -> Status,
}
