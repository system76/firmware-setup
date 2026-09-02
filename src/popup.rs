// SPDX-License-Identifier: GPL-3.0-only

//! HII Popup Protocol
//!
//! ## References
//!
//! - UEFI 2.11: 35.7 HII Popup Protocol

#![allow(unused)]

use std::prelude::*;
use std::uefi::hii::StringId;
use std::uefi::hii::database::HiiHandle;

// Protocol definition

/// `EFI_HII_POPUP_STYLE`
#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HiiPopupStyle(pub u32);

impl HiiPopupStyle {
    pub const INFO: Self = Self(0);
    pub const WARNING: Self = Self(1);
    pub const ERROR: Self = Self(2);
}

/// `EFI_HII_POPUP_TYPE`
#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HiiPopupType(pub u32);

impl HiiPopupType {
    pub const OK: Self = Self(0);
    pub const OK_CANCEL: Self = Self(1);
    pub const YES_NO: Self = Self(2);
    pub const YES_NO_CANCEL: Self = Self(3);
}

/// `EFI_HII_POPUP_SELECTION`
#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HiiPopupSelection(pub u32);

impl HiiPopupSelection {
    pub const OK: Self = Self(0);
    pub const CANCEL: Self = Self(1);
    pub const YES: Self = Self(2);
    pub const NO: Self = Self(3);
}

/// `EFI_HII_POPUP_PROTOCOL`
#[repr(C)]
#[derive(Debug)]
pub struct HiiPopupProtocol {
    /// Protocol revision
    pub revision: u64,
    /// Displays a popup window
    pub create_popup: unsafe extern "efiapi" fn(
        this: *mut Self,
        popup_style: HiiPopupStyle,
        popup_type: HiiPopupType,
        hii_handle: HiiHandle,
        message: StringId,
        user_selection: *mut HiiPopupSelection,
    ) -> Status,
}

impl HiiPopupProtocol {
    pub const GUID: Guid = guid!("4311edc0-6054-46d4-9e40-893ea952fccc");
    pub const REVISION: u64 = 1;
}

// Protocol implementation

extern "efiapi" fn create_popup(
    this: *mut HiiPopupProtocol,
    popup_style: HiiPopupStyle,
    popup_type: HiiPopupType,
    hii_handle: HiiHandle,
    message: StringId,
    user_selection: *mut HiiPopupSelection,
) -> Status {
    // - Check popup style/type are valid
    // - Get HII string to display (`EFI_HII_STRING_PROTOCOL.GetString()`)
    // - Draw popup with HII string and options
    //   - Calculate position based on HII string length and popup type
    // - Wait for user selection
    //   - Handle changing selections

    Status::UNSUPPORTED
}

pub static HII_POPUP: HiiPopupProtocol = HiiPopupProtocol {
    revision: HiiPopupProtocol::REVISION,
    create_popup,
};
