use core::ptr;
use std::prelude::*;
use std::proto::Protocol;

pub struct Rng(pub &'static mut RngProtocol);

impl Rng {
    pub fn read(&self, buf: &mut [u8]) -> Result<()> {
        unsafe {
            Result::from((self.0.get_rng)(
                self.0,
                ptr::null(),
                buf.len(),
                buf.as_mut_ptr(),
            ))?;
        }
        Ok(())
    }
}

impl Protocol<RngProtocol> for Rng {
    fn guid() -> Guid {
        RngProtocol::GUID
    }

    fn new(inner: &'static mut RngProtocol) -> Self {
        Rng(inner)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct RngProtocol {
    pub get_info: unsafe extern "efiapi" fn(
        &RngProtocol,
        RNGAlgorithmListSize: &mut usize,
        RNGAlgorithmList: *mut Guid,
    ) -> Status,
    pub get_rng: unsafe extern "efiapi" fn(
        &RngProtocol,
        RNGAlgorithm: *const Guid,
        RNGValueLength: usize,
        RNGValue: *mut u8,
    ) -> Status,
}

impl RngProtocol {
    pub const GUID: Guid = guid!("3152bca5-eade-433d-862e-c01cdc291f44");
}
