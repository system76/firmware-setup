use uefi;
use uefi_alloc;

use crate::main;

#[no_mangle]
pub extern "win64" fn _start(handle: uefi::Handle, uefi: &'static mut uefi::system::SystemTable) -> isize {
    unsafe {
        crate::HANDLE = handle;
        crate::UEFI = uefi;

        uefi_alloc::init(::core::mem::transmute(&mut *crate::UEFI));
    }

    main();

    0
}
