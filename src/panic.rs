// These functions are used by the compiler, but not
// for a bare-bones hello world. These are normally
// provided by libstd.
#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() {}

// This function may be needed based on the compilation target.
#[lang = "eh_unwind_resume"]
#[no_mangle]
pub extern fn rust_eh_unwind_resume() {
    loop {}
}

#[panic_handler]
#[no_mangle]
pub extern fn rust_begin_panic(pi: &::core::panic::PanicInfo) -> ! {
    print!("SETUP PANIC: {}", pi);

    loop {}
}

#[lang = "oom"]
#[no_mangle]
pub extern "C" fn rust_oom(layout: ::core::alloc::Layout) -> ! {
    println!(
        "SETUP OOM: {} bytes aligned to {} bytes\n",
        layout.size(),
        layout.align()
    );

    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn _Unwind_Resume() {
    loop {}
}
