// SPDX-License-Identifier: GPL-3.0-only

//! edk2 Form Display Engine (FDE) Protocol

pub mod custom;
#[macro_use]
pub mod list;

use list::*;
use std::prelude::*;
use std::uefi::hii::database::HiiHandle;
use std::uefi::hii::ifr::{HiiValue, IfrOneOfOption, IfrOpHeader};
use std::uefi::hii::{AnimationId, ImageId, StringId};
use std::uefi::text::TextInputKey;

// TODO: Move to FormBrowser2
#[repr(C)]
pub struct ScreenDescriptor {
    pub left_column: usize,
    pub right_column: usize,
    pub top_row: usize,
    pub bottom_row: usize,
}

// Custom browser actions defined by FDE.
pub const BROWSER_ACTION_NONE: u32 = 1 << 16;
pub const BROWSER_ACTION_FORM_EXIT: u32 = 1 << 17;

/// `DISPLAY_QUESTION_OPTION`
#[repr(C)]
pub struct QuestionOption {
    pub signature: usize,
    pub link: ListEntry<QuestionOption>,
    pub option_opcode_ptr: *const IfrOneOfOption,
    pub image_id: ImageId,
    pub animation_id: AnimationId,
}
list_entry!(QuestionOption, link);

#[allow(dead_code)]
impl QuestionOption {
    pub const SIGNATURE: u32 = u32::from_le_bytes(*b"QOPT");

    pub fn option_opcode(&self) -> Option<&IfrOneOfOption> {
        if self.option_opcode_ptr.is_null() {
            None
        } else {
            Some(unsafe { &*self.option_opcode_ptr })
        }
    }
}

/// `STATEMENT_ERROR_INFO`
#[repr(C)]
pub struct StatementErrorInfo {
    pub string_id: StringId,
    pub time_out: u8,
}

/// `FORM_DISPLAY_ENGINE_STATEMENT`
#[repr(C)]
pub struct Statement {
    pub signature: usize,
    pub version: usize,
    pub display_link: ListEntry<Statement>,
    pub opcode_ptr: *const IfrOpHeader,
    pub current_value: HiiValue,
    pub setting_changed_flag: bool,
    pub nest_statement_list: ListHead<Statement>,
    pub option_list_head: ListHead<QuestionOption>,
    pub attribute: u32,
    pub validate_question: unsafe extern "efiapi" fn(
        form: *const Form,
        statement: *const Self,
        value: *const HiiValue,
        error_info: *mut StatementErrorInfo,
    ) -> u32,
    pub password_check: unsafe extern "efiapi" fn(
        form: *const Form,
        statement: *const Self,
        password_string: *const u16,
    ) -> Status,
    pub image_id: ImageId,
    pub animation_id: AnimationId,
}
list_entry!(Statement, display_link);

#[allow(dead_code)]
impl Statement {
    pub const SIGNATURE: u32 = u32::from_le_bytes(*b"FSTA");
    pub const VERSION_1: usize = 0x1_0000;

    pub fn opcode(&self) -> Option<&IfrOpHeader> {
        if self.opcode_ptr.is_null() {
            None
        } else {
            Some(unsafe { &*self.opcode_ptr })
        }
    }
}

/// `BROWSER_HOT_KEY`
#[repr(C)]
pub struct HotKey {
    pub signature: usize,
    pub link: ListEntry<HotKey>,
    pub key_data: *const TextInputKey,
    pub action: u32,
    pub default_id: u16,
    pub help_string: *const u16,
}
list_entry!(HotKey, link);

#[allow(dead_code)]
impl HotKey {
    pub const SIGNATURE: u32 = u32::from_le_bytes(*b"BHKS");
}

/// `FORM_DISPLAY_ENGINE_FORM`
#[repr(C)]
pub struct Form {
    pub signature: usize,
    pub version: usize,
    pub statement_list_head: ListHead<Statement>,
    pub statement_list_osf: ListHead<Statement>,
    pub screen_dimensions: *const ScreenDescriptor,
    pub formset_guid: Guid,
    pub hii_handle: HiiHandle,
    pub form_id: u16,
    pub form_title: StringId,
    pub attribute: u32,
    pub setting_changed_flag: bool,
    pub highlighted_statement: *const Statement,
    pub form_refresh_event: Event,
    pub hotkey_list_head: ListHead<HotKey>,
    pub image_id: ImageId,
    pub animation_id: AnimationId,
    pub browser_status: u32,
    pub error_string: *const u16,
}

#[allow(dead_code)]
impl Form {
    pub const SIGNATURE: u32 = u32::from_le_bytes(*b"FFRM");
    pub const VERSION_1: usize = 0x1_0000;
}

/// `USER_INPUT`
#[repr(C)]
pub struct UserInput {
    pub selected_statement: *const Statement,
    pub input_value: HiiValue,
    pub action: u32,
    pub default_id: u16,
}

/// `EDKII_FORM_DISPLAY_ENGINE_PROTOCOL`
#[derive(Debug)]
#[repr(C)]
pub struct FormDisplayEngine {
    pub form_display: unsafe extern "efiapi" fn(
        form_data: *const Form,
        user_input_data: *mut UserInput,
    ) -> Status,
    pub exit_display: unsafe extern "efiapi" fn(),
    pub confirm_data_change: unsafe extern "efiapi" fn() -> usize,
}

impl FormDisplayEngine {
    pub const GUID: Guid = guid!("9bbe29e9-fda1-41ec-ad52-452213742d2e");
}
