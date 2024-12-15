// SPDX-License-Identifier: GPL-3.0-only

use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.ends_with("-unknown-uefi") {
        println!("cargo::rustc-link-arg=/heap:0,0");
        println!("cargo::rustc-link-arg=/stack:0,0");
        println!("cargo::rustc-link-arg=/dll");
        println!("cargo::rustc-link-arg=/base:0");
        println!("cargo::rustc-link-arg=/align:32");
        println!("cargo::rustc-link-arg=/filealign:32");
        println!("cargo::rustc-link-arg=/subsystem:efi_boot_service_driver");
    }
}
