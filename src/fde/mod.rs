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
    pub LeftColumn: usize,
    pub RightColumn: usize,
    pub TopRow: usize,
    pub BottomRow: usize,
}

// Custom browser actions defined by FDE.
pub const BROWSER_ACTION_NONE: u32 = 1 << 16;
pub const BROWSER_ACTION_FORM_EXIT: u32 = 1 << 17;

/// `DISPLAY_QUESTION_OPTION`
#[repr(C)]
pub struct QuestionOption {
    pub Signature: usize,
    pub Link: ListEntry<QuestionOption>,
    pub OptionOpCodePtr: *const IfrOneOfOption,
    pub ImageId: ImageId,
    pub AnimationId: AnimationId,
}
list_entry!(QuestionOption, Link);

#[allow(dead_code)]
impl QuestionOption {
    pub const SIGNATURE: u32 = u32::from_le_bytes(*b"QOPT");

    pub fn OptionOpCode(&self) -> Option<&IfrOneOfOption> {
        if self.OptionOpCodePtr.is_null() {
            None
        } else {
            Some(unsafe { &*self.OptionOpCodePtr })
        }
    }
}

/// `STATEMENT_ERROR_INFO`
#[repr(C)]
pub struct StatementErrorInfo {
    pub StringId: StringId,
    pub TimeOut: u8,
}

/// `FORM_DISPLAY_ENGINE_STATEMENT`
#[repr(C)]
pub struct Statement {
    pub Signature: usize,
    pub Version: usize,
    pub DisplayLink: ListEntry<Statement>,
    pub OpCodePtr: *const IfrOpHeader,
    pub CurrentValue: HiiValue,
    pub SettingChangedFlag: bool,
    pub NestStatementList: ListHead<Statement>,
    pub OptionListHead: ListHead<QuestionOption>,
    pub Attribute: u32,
    pub ValidateQuestion: unsafe extern "efiapi" fn(
        form: *const Form,
        statement: *const Self,
        value: *const HiiValue,
        error_info: *mut StatementErrorInfo,
    ) -> u32,
    pub PasswordCheck: unsafe extern "efiapi" fn(
        form: *const Form,
        statement: *const Self,
        password_string: *const u16,
    ) -> Status,
    pub ImageId: ImageId,
    pub AnimationId: AnimationId,
}
list_entry!(Statement, DisplayLink);

#[allow(dead_code)]
impl Statement {
    pub const SIGNATURE: u32 = u32::from_le_bytes(*b"FSTA");
    pub const VERSION_1: usize = 0x1_0000;

    pub fn OpCode(&self) -> Option<&IfrOpHeader> {
        if self.OpCodePtr.is_null() {
            None
        } else {
            Some(unsafe { &*self.OpCodePtr })
        }
    }
}

/// `BROWSER_HOT_KEY`
#[repr(C)]
pub struct HotKey {
    pub Signature: usize,
    pub Link: ListEntry<HotKey>,
    pub KeyData: *const TextInputKey,
    pub Action: u32,
    pub DefaultId: u16,
    pub HelpString: *const u16,
}
list_entry!(HotKey, Link);

#[allow(dead_code)]
impl HotKey {
    pub const SIGNATURE: u32 = u32::from_le_bytes(*b"BHKS");
}

/// `FORM_DISPLAY_ENGINE_FORM`
#[repr(C)]
pub struct Form {
    pub Signature: usize,
    pub Version: usize,
    pub StatementListHead: ListHead<Statement>,
    pub StatementListOSF: ListHead<Statement>,
    pub ScreenDimensions: *const ScreenDescriptor,
    pub FormSetGuid: Guid,
    pub HiiHandle: HiiHandle,
    pub FormId: u16,
    pub FormTitle: StringId,
    pub Attribute: u32,
    pub SettingChangedFlag: bool,
    pub HighLightedStatement: *const Statement,
    pub FormRefreshEvent: Event,
    pub HotKeyListHead: ListHead<HotKey>,
    pub ImageId: ImageId,
    pub AnimationId: AnimationId,
    pub BrowserStatus: u32,
    pub ErrorString: *const u16,
}

#[allow(dead_code)]
impl Form {
    pub const SIGNATURE: u32 = u32::from_le_bytes(*b"FFRM");
    pub const VERSION_1: usize = 0x1_0000;
}

/// `USER_INPUT`
#[repr(C)]
pub struct UserInput {
    pub SelectedStatement: *const Statement,
    pub InputValue: HiiValue,
    pub Action: u32,
    pub DefaultId: u16,
}

/// `EDKII_FORM_DISPLAY_ENGINE_PROTOCOL`
#[derive(Debug)]
#[repr(C)]
pub struct FormDisplayEngine {
    pub FormDisplay: unsafe extern "efiapi" fn(
        form_data: *const Form,
        user_input_data: *mut UserInput,
    ) -> Status,
    pub ExitDisplay: unsafe extern "efiapi" fn(),
    pub ConfirmDataChange: unsafe extern "efiapi" fn() -> usize,
}

impl FormDisplayEngine {
    pub const GUID: Guid = guid!("9bbe29e9-fda1-41ec-ad52-452213742d2e");
}
