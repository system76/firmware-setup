use uefi;
use uefi_alloc;

use main;

#[no_mangle]
pub extern "win64" fn _start(handle: uefi::Handle, uefi: &'static mut uefi::system::SystemTable) -> isize {
    unsafe {
        ::HANDLE = handle;
        ::UEFI = uefi;

        uefi_alloc::init(::core::mem::transmute(&mut *::UEFI));
    }

    main();

    0
}
