// SPDX-License-Identifier: GPL-3.0-only

// TODO: Move to uefi library

use super::font::FontInfo;
use std::prelude::*;
use std::proto::Protocol;
use std::uefi::hii::StringId;
use std::uefi::hii::database::HiiHandle;

/// `EFI_HII_STRING_PROTOCOL`
#[repr(C)]
#[derive(Debug)]
pub struct HiiStringProtocol {
    pub new_string: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        string_id: *mut StringId,
        language: *const u8,
        language_name: *const u16,
        string: *const u16,
        string_font_info: *const FontInfo,
    ) -> Status,
    pub get_string: unsafe extern "efiapi" fn(
        this: *const Self,
        language: *const u8,
        package_list: HiiHandle,
        string_id: StringId,
        string: *mut u16,
        string_size: *mut usize,
        string_font_info: *mut *mut FontInfo,
    ) -> Status,
    pub set_string: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        string_id: StringId,
        language: *const u8,
        string: *const u16,
        string_font_info: *const FontInfo,
    ) -> Status,
    pub get_languages: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        languages: *mut u8,
        languages_size: *mut usize,
    ) -> Status,
    pub get_secondary_languages: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: HiiHandle,
        primary_language: *const u8,
        secondary_languages: *mut u8,
        secondary_languages_size: *mut usize,
    ) -> Status,
}

impl HiiStringProtocol {
    pub const GUID: Guid = guid!("0fd96974-23aa-4cdc-b9cb-98d17750322a");

    pub fn string(&self, package_list: HiiHandle, string_id: StringId) -> Result<String> {
        let mut data = vec![0u16; 4096];
        let mut len = data.len();
        unsafe {
            Result::from((self.get_string)(
                self,
                c"en-US".as_ptr() as *const u8,
                package_list,
                string_id,
                data.as_mut_ptr(),
                &mut len,
                core::ptr::null_mut(),
            ))?;
        }
        data.truncate(len);

        let mut string = String::new();
        for &w in data.iter() {
            if w == 0 {
                break;
            }
            let c = unsafe { core::char::from_u32_unchecked(w as u32) };
            string.push(c);
        }
        Ok(string)
    }
}

impl Protocol<HiiStringProtocol> for &'static mut HiiStringProtocol {
    fn guid() -> Guid {
        HiiStringProtocol::GUID
    }

    fn new(inner: &'static mut HiiStringProtocol) -> Self {
        inner
    }
}
