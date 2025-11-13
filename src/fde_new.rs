// SPDX-License-Identifier: GPL-3.0-only
// SPDX-FileCopyrightText: 2025 System76, Inc.

use crate::edk2::fde::FormDisplayEngine;
use crate::edk2::fde::FormDisplayEngineForm;
use crate::edk2::fde::FormDisplayEngineProtocol;
use crate::edk2::fde::UserInput;
use uefi::Result;
use uefi::prelude::*;

extern "efiapi" fn form_display(
    _form_data: *const FormDisplayEngineForm,
    _user_input_data: *mut UserInput,
) -> uefi_raw::Status {
    uefi_raw::Status::UNSUPPORTED
}

extern "efiapi" fn exit_display() {}

extern "efiapi" fn confirm_data_change() -> usize {
    0
}

static CUSTOM_FDE: FormDisplayEngineProtocol = FormDisplayEngineProtocol {
    form_display,
    exit_display,
    confirm_data_change,
};

pub fn replace_interface() -> Result<()> {
    let handle = boot::get_handle_for_protocol::<FormDisplayEngine>()?;
    let old = unsafe {
        boot::open_protocol::<FormDisplayEngine>(
            boot::OpenProtocolParams {
                handle,
                agent: boot::image_handle(),
                controller: None,
            },
            boot::OpenProtocolAttributes::GetProtocol,
        )?
    };

    unsafe {
        boot::reinstall_protocol_interface(
            handle,
            &FormDisplayEngineProtocol::GUID,
            core::ptr::addr_of!(*old.get().unwrap()).cast(),
            core::ptr::addr_of!(CUSTOM_FDE).cast(),
        )
    }
}
