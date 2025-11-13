// SPDX-License-Identifier: GPL-3.0-only
// SPDX-FileCopyrightText: 2025 System76, Inc.

//! Form Display Engine (FDE) protocol

use uefi::proto::unsafe_protocol;
use uefi_raw::Boolean;
use uefi_raw::Char16;
use uefi_raw::Event;
use uefi_raw::Status;
use uefi_raw::protocol::hii::AnimationId;
use uefi_raw::protocol::hii::ImageId;
use uefi_raw::protocol::hii::form_browser::ScreenDescriptor;
use uefi_raw::protocol::hii::ifr::IfrOpHeader;
use uefi_raw::protocol::hii::ifr::IfrTypeValue;
use uefi_raw::protocol::hii::{HiiHandle, StringId};
use uefi_raw::{Guid, guid};

// TODO
#[derive(Debug)]
#[repr(C)]
pub struct ListEntry {}

/// EFI_HII_VALUE
#[derive(Debug)]
#[repr(C)]
pub struct HiiValue {
    pub hii_type: u8,
    pub buffer: *mut u8,
    pub buffer_len: u16,
    pub value: IfrTypeValue,
}

/// FORM_DISPLAY_ENGINE_STATEMENT
//#[derive(Debug)]
#[repr(C)]
pub struct FormDisplayEngineStatement {
    pub signature: usize,
    pub version: usize,
    pub display_link: ListEntry,
    pub opcode: IfrOpHeader,
}

/// FORM_DISPLAY_ENGINE_FORM
#[derive(Debug)]
#[repr(C)]
pub struct FormDisplayEngineForm {
    pub signature: usize,
    pub version: usize,
    pub statement_list_gead: ListEntry,
    pub statement_list_osf: ListEntry,
    pub screen_dimentsions: *mut ScreenDescriptor,
    pub formset_guid: Guid,
    pub hii_handle: HiiHandle,
    pub form_id: u16,
    pub form_title: StringId,
    pub attribute: u32,
    pub setting_changed_flag: Boolean,
    pub highlighted_statement: *mut FormDisplayEngineStatement,
    pub form_refresh_event: Event,
    pub hotkey_list_head: ListEntry,
    pub image_id: ImageId,
    pub animation_id: AnimationId,
    pub browser_status: u32,
    pub error_string: *mut Char16,
}

/// USER_INPUT
#[derive(Debug)]
#[repr(C)]
pub struct UserInput {
    pub selected_statement: *mut FormDisplayEngineStatement,
    pub input_value: HiiValue,
    pub action: u32,
    pub default_id: u16,
}

/// EDKII_FORM_DISPLAY_ENGINE_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct FormDisplayEngineProtocol {
    pub form_display: unsafe extern "efiapi" fn(
        form_data: *const FormDisplayEngineForm,
        user_input_data: *mut UserInput,
    ) -> Status,
    pub exit_display: unsafe extern "efiapi" fn(),
    pub confirm_data_change: unsafe extern "efiapi" fn() -> usize,
}

impl FormDisplayEngineProtocol {
    pub const GUID: Guid = guid!("9bbe29e9-fda1-41ec-ad52-452213742d2e");
}

#[unsafe_protocol(FormDisplayEngineProtocol::GUID)]
#[repr(transparent)]
pub struct FormDisplayEngine(pub FormDisplayEngineProtocol);
