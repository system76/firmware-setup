// SPDX-License-Identifier: GPL-3.0-only

// TODO: Move to uefi library

bitflags::bitflags! {
    /// `EFI_HII_FONT_STYLE`
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(transparent)]
    pub struct HiiFontStyle: u32 {
        const BOLD = 1 << 0;
        const ITALIC = 1 << 1;
        const EMBOSS = 1 << 16;
        const OUTLINE = 1 << 17;
        const SHADOW = 1 << 18;
        const UNDERLINE = 1 << 19;
        const DBL_UNDER = 1 << 20;
    }
}

impl HiiFontStyle {
    pub const NORMAL: Self = Self::empty();
}

/// `EFI_FONT_INFO`
#[repr(C)]
#[derive(Debug)]
pub struct FontInfo {
    pub font_style: HiiFontStyle,
    pub font_size: u16,
    pub font_name: [u16; 0],
}
