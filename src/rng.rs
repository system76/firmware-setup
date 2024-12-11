use core::ptr;
use std::proto::Protocol;
use std::prelude::*;

pub const RNG_PROTOCOL_GUID: Guid = guid!("3152bca5-eade-433d-862e-c01cdc291f44");

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
