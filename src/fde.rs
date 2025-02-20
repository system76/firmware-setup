// SPDX-License-Identifier: GPL-3.0-only

use core::{char, cmp, mem, ptr, slice};
use orbclient::{Color, Renderer};
use orbfont::Text;
use std::ffi;
use std::prelude::*;
use std::proto::Protocol;
use std::uefi::hii::database::HiiHandle;
use std::uefi::hii::ifr::{
    HiiValue, IfrAction, IfrCheckbox, IfrNumeric, IfrOneOf, IfrOneOfOption, IfrOpCode, IfrOpHeader,
    IfrOrderedList, IfrRef, IfrStatementHeader, IfrSubtitle, IfrTypeValueEnum,
};
use std::uefi::hii::{AnimationId, ImageId, StringId};
use std::uefi::text::TextInputKey;

use crate::display::{Display, Output};
use crate::key::{raw_key, Key};
use crate::ui::Ui;

// TODO: Move to uefi library {
pub const HII_STRING_PROTOCOL_GUID: Guid = guid!("0fd96974-23aa-4cdc-b9cb-98d17750322a");

#[repr(C)]
pub struct HiiStringProtocol {
    pub NewString: extern "efiapi" fn(), //TODO
    pub GetString: extern "efiapi" fn(
        &HiiStringProtocol,
        Language: *const u8,
        PackageList: HiiHandle,
        StringId: StringId,
        String: *mut u16,
        StringSize: &mut usize,
        StringFontInfo: usize, // TODO
    ) -> Status,
    pub SetString: extern "efiapi" fn(),             //TODO
    pub GetLanguages: extern "efiapi" fn(),          //TODO
    pub GetSecondaryLanguages: extern "efiapi" fn(), //TODO
}

impl HiiStringProtocol {
    pub fn string(&self, PackageList: HiiHandle, StringId: StringId) -> Result<String> {
        let mut data = vec![0u16; 4096];
        let mut len = data.len();
        Result::from((self.GetString)(
            self,
            b"en-US\0".as_ptr(),
            PackageList,
            StringId,
            data.as_mut_ptr(),
            &mut len,
            0,
        ))?;
        data.truncate(len);

        let mut string = String::new();
        for &w in data.iter() {
            if w == 0 {
                break;
            }
            let c = unsafe { char::from_u32_unchecked(w as u32) };
            string.push(c);
        }
        Ok(string)
    }
}

impl Protocol<HiiStringProtocol> for &'static mut HiiStringProtocol {
    fn guid() -> Guid {
        HII_STRING_PROTOCOL_GUID
    }

    fn new(inner: &'static mut HiiStringProtocol) -> Self {
        inner
    }
}

// } TODO: Move to uefi library

// TODO: move to uefi library {
#[repr(C)]
pub struct ListEntry<T> {
    Flink: *mut ListEntry<T>,
    Blink: *mut ListEntry<T>,
}

#[allow(dead_code)]
impl<T> ListEntry<T> {
    pub fn previous(&self) -> Option<&Self> {
        if self.Blink.is_null() {
            None
        } else {
            Some(unsafe { &*self.Blink })
        }
    }

    pub fn previous_mut(&mut self) -> Option<&mut Self> {
        if self.Blink.is_null() {
            None
        } else {
            Some(unsafe { &mut *self.Blink })
        }
    }

    pub fn next(&self) -> Option<&Self> {
        if self.Flink.is_null() {
            None
        } else {
            Some(unsafe { &*self.Flink })
        }
    }

    pub fn next_mut(&mut self) -> Option<&mut Self> {
        if self.Flink.is_null() {
            None
        } else {
            Some(unsafe { &mut *self.Flink })
        }
    }

    unsafe fn object_at(&self, offset: usize) -> &T {
        let addr = self as *const Self as usize;
        unsafe { &*((addr - offset) as *const T) }
    }

    unsafe fn object_at_mut(&mut self, offset: usize) -> &mut T {
        let addr = self as *mut Self as usize;
        unsafe { &mut *((addr - offset) as *mut T) }
    }
}

pub trait ListEntryObject<T> {
    unsafe fn object(&self) -> &T;

    #[allow(dead_code)]
    unsafe fn object_mut(&mut self) -> &mut T;
}

macro_rules! list_entry {
    ($t:ident, $l:tt) => {
        impl ListEntryObject<$t> for ListEntry<$t> {
            unsafe fn object(&self) -> &$t {
                unsafe { self.object_at(offset_of!($t, $l)) }
            }

            unsafe fn object_mut(&mut self) -> &mut $t {
                unsafe { self.object_at_mut(offset_of!($t, $l)) }
            }
        }
    };
}

pub struct ListEntryIter<'a, T> {
    start: Option<&'a ListEntry<T>>,
    current: Option<&'a ListEntry<T>>,
}

impl<'a, T> Iterator for ListEntryIter<'a, T>
where
    ListEntry<T>: ListEntryObject<T>,
{
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;
        let next = current.next();
        if next.map(|x| x as *const _) == self.start.map(|x| x as *const _) {
            self.current = None;
            return None;
        } else {
            self.current = next;
        }
        Some(unsafe { current.object() })
    }
}

#[repr(transparent)]
pub struct ListHead<T>(ListEntry<T>);

impl<T> ListHead<T> {
    pub fn iter(&self) -> ListEntryIter<T> {
        let next = self.0.next();
        ListEntryIter {
            start: next,
            current: next,
        }
    }
}
// } TODO: move to uefi library

#[repr(C)]
pub struct QuestionOption {
    pub Signature: usize,
    pub Link: ListEntry<QuestionOption>,
    pub OptionOpCodePtr: *const IfrOneOfOption,
    pub ImageId: ImageId,
    pub AnimationId: AnimationId,
}
list_entry!(QuestionOption, Link);

impl QuestionOption {
    pub fn OptionOpCode(&self) -> Option<&IfrOneOfOption> {
        if self.OptionOpCodePtr.is_null() {
            None
        } else {
            Some(unsafe { &*self.OptionOpCodePtr })
        }
    }
}

#[repr(C)]
pub struct StatementErrorInfo {
    pub StringId: StringId,
    pub TimeOut: u8,
}

pub type ValidateQuestion = extern "efiapi" fn(
    Form: &Form,
    Statement: &Statement,
    Value: &HiiValue,
    ErrorInfo: &mut StatementErrorInfo,
) -> u32;

pub type PasswordCheck =
    extern "efiapi" fn(Form: &Form, Statement: &Statement, PasswordString: *const u16) -> Status;

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
    pub ValidateQuestion: Option<ValidateQuestion>,
    pub PasswordCheck: Option<PasswordCheck>,
    pub ImageId: ImageId,
    pub AnimationId: AnimationId,
}
list_entry!(Statement, DisplayLink);

impl Statement {
    pub fn OpCode(&self) -> Option<&IfrOpHeader> {
        if self.OpCodePtr.is_null() {
            None
        } else {
            Some(unsafe { &*self.OpCodePtr })
        }
    }
}

#[repr(C)]
pub struct ScreenDescriptor {
    pub LeftColumn: usize,
    pub RightColumn: usize,
    pub TopRow: usize,
    pub BottomRow: usize,
}

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

const FRONT_PAGE_FORM_ID: u16 = 0x7600;

const BROWSER_ACTION_NONE: u32 = 1 << 16;
const BROWSER_ACTION_FORM_EXIT: u32 = 1 << 17;

#[repr(C)]
pub struct UserInput {
    pub SelectedStatement: *const Statement,
    pub InputValue: HiiValue,
    pub Action: u32,
    pub DefaultId: u16,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct Fde {
    pub FormDisplay: extern "efiapi" fn(FormData: &Form, UserInputData: &mut UserInput) -> Status,
    pub ExitDisplay: extern "efiapi" fn(),
    pub ConfirmDataChange: extern "efiapi" fn() -> usize,
}

static mut DISPLAY: *mut Display = ptr::null_mut();

#[allow(dead_code)]
struct ElementOption<'a> {
    option_ptr: *const QuestionOption,
    prompt: Text<'a>,
    value: IfrTypeValueEnum,
}

struct Element<'a> {
    statement_ptr: *const Statement,
    prompt: String,
    help: String,
    value: IfrTypeValueEnum,
    options: Vec<ElementOption<'a>>,
    selectable: bool,
    editable: bool,
    list: bool,
    list_i: usize,
    buffer_opt: Option<&'static mut [u8]>,
}

#[derive(PartialEq)]
enum EventType {
    Driver,
    Keyboard,
}

fn wait_for_events(form: &Form) -> Result<EventType> {
    let uefi = std::system_table();
    let mut index = 0;
    let mut events = vec![uefi.ConsoleIn.WaitForKey];

    if form.FormRefreshEvent != Event(0) {
        events.push(form.FormRefreshEvent);
    }

    Result::from((uefi.BootServices.WaitForEvent)(
        events.len(),
        events.as_mut_ptr(),
        &mut index,
    ))?;

    if index == 0 {
        Ok(EventType::Keyboard)
    } else {
        Ok(EventType::Driver)
    }
}

#[allow(unused_assignments)]
fn form_display_inner(form: &Form, user_input: &mut UserInput) -> Result<()> {
    let hii_string = <&'static mut HiiStringProtocol>::one()?;

    let string =
        |string_id: StringId| -> Result<String> { hii_string.string(form.HiiHandle, string_id) };

    let display: &mut Display = unsafe {
        if DISPLAY.is_null() {
            let display = Display::new(Output::one()?);
            DISPLAY = Box::into_raw(Box::new(display));
        }
        &mut *DISPLAY
    };

    let (display_w, display_h) = (display.width(), display.height());

    let scale = if display_h > 1440 {
        4
    } else if display_h > 720 {
        2
    } else {
        1
    };

    // Style {
    let margin_lr = 8 * scale;
    let margin_tb = 4 * scale;

    let title_font_size = (20 * scale) as f32;
    let font_size = (16 * scale) as f32; // (display_h as f32) / 26.0
    let help_font_size = (12 * scale) as f32;
    // } Style

    let ui = Ui::new()?;

    'render: loop {
        let mut hotkey_helps = Vec::new();
        for hotkey in form.HotKeyListHead.iter() {
            let hotkey_help = ffi::nstr(hotkey.HelpString);
            hotkey_helps.push(hotkey_help);
        }

        let mut selected = !0;
        let mut editing = false;
        let mut elements = Vec::new();
        for statement in form.StatementListHead.iter() {
            let statement_ptr = statement as *const _;

            let mut options = Vec::new();
            for option in statement.OptionListHead.iter() {
                let option_ptr = option as *const _;
                if let Some(op) = option.OptionOpCode() {
                    let value = unsafe { op.Value.to_enum(op.Kind) };
                    let prompt = ui
                        .font
                        .render(&string(op.Option).unwrap_or_default(), font_size);
                    options.push(ElementOption {
                        option_ptr,
                        prompt,
                        value,
                    });
                }
            }

            let add_element =
                |header: IfrStatementHeader, selectable: bool, editable: bool, list: bool| {
                    let value = unsafe {
                        statement
                            .CurrentValue
                            .Value
                            .to_enum(statement.CurrentValue.Kind)
                    };
                    let buffer_opt = if statement.CurrentValue.Buffer.is_null() {
                        None
                    } else {
                        let buffer = unsafe {
                            slice::from_raw_parts_mut(
                                statement.CurrentValue.Buffer,
                                statement.CurrentValue.BufferLen as usize,
                            )
                        };
                        // Order list according to buffer
                        if list {
                            let mut offset = 0;
                            for i in 0..options.len() {
                                for j in i..options.len() {
                                    macro_rules! check_option {
                                        ($x:ident) => {{
                                            let next_offset = offset + mem::size_of_val(&$x);
                                            if next_offset <= buffer.len() {
                                                let mut x_copy = $x;
                                                unsafe {
                                                    ptr::copy(
                                                        buffer.as_ptr().add(offset) as *const _,
                                                        &mut x_copy,
                                                        1,
                                                    );
                                                };
                                                if $x == x_copy {
                                                    offset = next_offset;
                                                    true
                                                } else {
                                                    false
                                                }
                                            } else {
                                                false
                                            }
                                        }};
                                    }
                                    let matches = match options[j].value {
                                        IfrTypeValueEnum::U8(u8) => check_option!(u8),
                                        IfrTypeValueEnum::U16(u16) => check_option!(u16),
                                        IfrTypeValueEnum::U32(u32) => check_option!(u32),
                                        IfrTypeValueEnum::U64(u64) => check_option!(u64),
                                        _ => false,
                                    };
                                    if matches {
                                        if i != j {
                                            options.swap(i, j);
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        Some(buffer)
                    };
                    if statement_ptr == form.HighLightedStatement || (selected == !0 && selectable)
                    {
                        selected = elements.len();
                    }
                    elements.push(Element {
                        statement_ptr,
                        prompt: string(header.Prompt).unwrap_or_default(),
                        help: string(header.Help).unwrap_or_default(),
                        value,
                        options,
                        selectable,
                        editable,
                        list,
                        list_i: 0,
                        buffer_opt,
                    });
                };

            if let Some(op) = statement.OpCode() {
                macro_rules! cast {
                    ($type:ty) => {{
                        op.cast::<$type>()
                    }};
                }
                match op.OpCode {
                    IfrOpCode::Action => {
                        if let Some(action) = unsafe { cast!(IfrAction) } {
                            add_element(action.QuestionHeader.Header, true, false, false);
                        }
                    }
                    IfrOpCode::Checkbox => {
                        if let Some(checkbox) = unsafe { cast!(IfrCheckbox) } {
                            add_element(checkbox.Question.Header, true, true, false);
                        }
                    }
                    IfrOpCode::Numeric => {
                        if let Some(numeric) = unsafe { cast!(IfrNumeric) } {
                            add_element(numeric.Question.Header, true, true, false);
                        }
                    }
                    IfrOpCode::OneOf => {
                        if let Some(one_of) = unsafe { cast!(IfrOneOf) } {
                            add_element(one_of.Question.Header, true, true, false);
                        }
                    }
                    IfrOpCode::OrderedList => {
                        if let Some(ordered_list) = unsafe { cast!(IfrOrderedList) } {
                            add_element(ordered_list.Question.Header, true, true, true);
                        }
                    }
                    IfrOpCode::Ref => {
                        if let Some(ref_) = unsafe { cast!(IfrRef) } {
                            add_element(ref_.Question.Header, true, false, false);
                        }
                    }
                    IfrOpCode::Subtitle => {
                        if let Some(subtitle) = unsafe { cast!(IfrSubtitle) } {
                            add_element(subtitle.Statement, false, false, false);
                        }
                    }
                    _ => (),
                }
            }
        }

        let title_opt = string(form.FormTitle).ok();
        let mut element_start = 0;
        'display: loop {
            display.set(ui.background_color);

            let draw_value_box = |display: &mut Display,
                                  x: i32,
                                  y: i32,
                                  value: &IfrTypeValueEnum,
                                  highlighted: bool|
             -> i32 {
                //TODO: Do not format in drawing loop
                let value_string = match value {
                    IfrTypeValueEnum::U8(value) => format!("{value}"),
                    IfrTypeValueEnum::U16(value) => format!("{value}"),
                    IfrTypeValueEnum::U32(value) => format!("{value}"),
                    IfrTypeValueEnum::U64(value) => format!("{value}"),
                    IfrTypeValueEnum::Bool(value) => {
                        return ui.draw_check_box(display, x, y, *value)
                    }
                    other => format!("{other:?}"),
                };

                // TODO: Do not render in drawing loop
                let rendered = ui.font.render(&value_string, font_size);
                ui.draw_text_box(display, x, y, &rendered, true, highlighted);
                rendered.height() as i32
            };

            let draw_options_box =
                |display: &mut Display, x: i32, mut y: i32, element: &Element| {
                    let mut w = 0;
                    for option in element.options.iter() {
                        w = cmp::max(w, option.prompt.width());
                    }

                    let start_y = y;
                    for (i, option) in element.options.iter().enumerate() {
                        let highlighted = i == element.list_i;
                        if highlighted && editing {
                            ui.draw_pretty_box(display, x, y, w, option.prompt.height(), true);
                        }
                        let text_color = if highlighted && editing {
                            ui.highlight_text_color
                        } else {
                            ui.text_color
                        };
                        option.prompt.draw(display, x, y, text_color);
                        y += option.prompt.height() as i32 + margin_tb;
                    }
                    if y > start_y {
                        ui.draw_pretty_box(
                            display,
                            x,
                            start_y,
                            w,
                            (y - start_y - margin_tb) as u32,
                            false,
                        );
                    }

                    y
                };

            let mut y = margin_tb;
            let mut bottom_y = display_h as i32;

            let (editing_list, editing_value) = elements
                .get(selected)
                .map(|e| (e.list, e.options.is_empty()))
                .unwrap_or((false, false));

            // Draw header
            if let Some(ref title) = title_opt {
                // TODO: Do not render in drawing loop
                let rendered = ui.font.render(title, title_font_size);
                let x = (display_w as i32 - rendered.width() as i32) / 2;
                ui.draw_text_box(display, x, y, &rendered, false, false);
                y += rendered.height() as i32 + margin_tb;
            }

            display.rect(0, y, display_w, 1, Color::rgb(0xac, 0xac, 0xac));
            y += margin_tb * 2;

            // Draw footer
            {
                let mut i = 0;
                let mut render_hotkey_help = |help: &str| {
                    let rendered = ui.font.render(help, help_font_size);
                    let x = match i % 3 {
                        0 => {
                            bottom_y -= rendered.height() as i32 + margin_tb;
                            (display_w as i32) * 2 / 3 + margin_lr
                        }
                        1 => (display_w as i32) / 3 + margin_lr,
                        _ => margin_lr,
                    };
                    ui.draw_text_box(display, x, bottom_y, &rendered, false, false);
                    i += 1;
                };

                if editing {
                    render_hotkey_help("Esc=Discard Changes");
                } else if form.FormId == FRONT_PAGE_FORM_ID {
                    render_hotkey_help("");
                } else {
                    render_hotkey_help("Esc=Exit");
                }
                if selected == !0 {
                    render_hotkey_help("");
                } else if editing {
                    render_hotkey_help("Enter=Save Changes");
                } else {
                    render_hotkey_help("Enter=Select Entry");
                }
                if selected == !0 {
                    render_hotkey_help("");
                } else if !editing || !editing_value {
                    render_hotkey_help("↑↓=Move Highlight");
                }

                if editing {
                    if editing_list {
                        render_hotkey_help("PgDn=Move Selection Down");
                        render_hotkey_help("");
                        render_hotkey_help("PgUp=Move Selection Up");
                    }
                } else {
                    for hotkey_help in hotkey_helps.iter() {
                        render_hotkey_help(hotkey_help);
                    }
                }

                bottom_y -= margin_tb * 3 / 2;
                display.rect(0, bottom_y, display_w, 1, Color::rgb(0xac, 0xac, 0xac));

                if let Some(element) = elements.get(selected) {
                    if !element.help.trim().is_empty() {
                        let rendered = ui.font.render(&element.help, help_font_size);
                        let x = (display_w as i32 - rendered.width() as i32) / 2;
                        bottom_y -= rendered.height() as i32 + margin_tb;
                        ui.draw_text_box(display, x, bottom_y, &rendered, false, false);

                        bottom_y -= margin_tb * 3 / 2;
                        display.rect(0, bottom_y, display_w, 1, Color::rgb(0xac, 0xac, 0xac));
                    }
                }
            }

            // Draw body
            let max_form_elements = ((bottom_y - y) / (font_size as i32 + margin_tb)) as usize;

            if element_start > 0 {
                // Draw up arrow to indicate more items above
                let arrow = ui.font.render("↑", help_font_size);
                ui.draw_text_box(
                    display,
                    (display_w - arrow.width()) as i32 - margin_lr,
                    y,
                    &arrow,
                    false,
                    false,
                );
            }

            for i in element_start..(element_start + max_form_elements) {
                if let Some(element) = elements.get(i) {
                    let highlighted = i == selected;
                    // TODO: Do not render in drawing loop
                    let mut h = 0;
                    for line in element.prompt.lines() {
                        let rendered = ui.font.render(line, font_size);
                        ui.draw_text_box(
                            display,
                            margin_lr,
                            y + h,
                            &rendered,
                            highlighted && !editing,
                            highlighted && !editing,
                        );
                        h += rendered.height() as i32;
                    }
                    if h == 0 {
                        h = font_size as i32;
                    }

                    let x = display_w as i32 / 2;
                    if element.list {
                        y = draw_options_box(display, x, y, element);
                        y -= h + margin_tb;
                    } else if let Some(option) =
                        element.options.iter().find(|o| o.value == element.value)
                    {
                        ui.draw_text_box(
                            display,
                            x,
                            y,
                            &option.prompt,
                            true,
                            highlighted && editing,
                        );
                    } else if element.editable {
                        draw_value_box(display, x, y, &element.value, highlighted && editing);
                    }

                    y += h + margin_tb;
                }
            }

            if elements.len() > max_form_elements
                && element_start < elements.len() - max_form_elements
            {
                // Draw down arrow to indicate more items below
                let arrow = ui.font.render("↓", help_font_size);
                ui.draw_text_box(
                    display,
                    (display_w - arrow.width()) as i32 - margin_lr,
                    bottom_y - arrow.height() as i32 - margin_tb * 2,
                    &arrow,
                    false,
                    false,
                );
            }

            display.sync();

            let signaled = wait_for_events(form)?;
            if signaled == EventType::Driver {
                user_input.Action = BROWSER_ACTION_NONE;
                break 'render;
            }

            // Consume all queued key presses
            'input: loop {
                let raw_key = match raw_key(false) {
                    Ok(ok) => ok,
                    Err(err) => match err {
                        Status::NOT_READY => break 'input,
                        _ => return Err(err),
                    },
                };

                if !editing {
                    for hotkey in form.HotKeyListHead.iter() {
                        let key_data = unsafe { &*hotkey.KeyData };
                        if key_data.ScanCode == raw_key.ScanCode
                            && key_data.UnicodeChar == raw_key.UnicodeChar
                        {
                            user_input.Action = hotkey.Action;
                            user_input.DefaultId = hotkey.DefaultId;
                            break 'render;
                        }
                    }
                }

                let key = Key::from(raw_key);
                match key {
                    Key::Enter => {
                        if let Some(element) = elements.get_mut(selected) {
                            let mut checkbox = false;
                            {
                                let statement = unsafe { &(*element.statement_ptr) };
                                if let Some(op) = statement.OpCode() {
                                    #[allow(clippy::single_match)]
                                    match op.OpCode {
                                        IfrOpCode::Checkbox => checkbox = true,
                                        _ => (),
                                    }
                                }
                            }

                            if checkbox {
                                if let IfrTypeValueEnum::Bool(b) = element.value {
                                    user_input.SelectedStatement = element.statement_ptr;
                                    unsafe {
                                        ptr::copy(
                                            &(*element.statement_ptr).CurrentValue,
                                            &mut user_input.InputValue,
                                            1,
                                        );
                                    }

                                    let (kind, value) =
                                        unsafe { IfrTypeValueEnum::Bool(!b).to_union() };
                                    user_input.InputValue.Kind = kind;
                                    user_input.InputValue.Value = value;

                                    break 'render;
                                }
                            } else if element.editable && !editing {
                                editing = true;
                            } else {
                                user_input.SelectedStatement = element.statement_ptr;
                                unsafe {
                                    ptr::copy(
                                        &(*element.statement_ptr).CurrentValue,
                                        &mut user_input.InputValue,
                                        1,
                                    );
                                }
                                if editing {
                                    if element.list {
                                        let mut offset = 0;
                                        if let Some(ref mut buffer) = element.buffer_opt {
                                            for option in element.options.iter() {
                                                macro_rules! copy_option {
                                                    ($x:ident) => {{
                                                        let next_offset =
                                                            offset + mem::size_of_val(&$x);
                                                        if next_offset <= buffer.len() {
                                                            unsafe {
                                                                ptr::copy(
                                                                    &$x,
                                                                    buffer.as_mut_ptr().add(offset)
                                                                        as *mut _,
                                                                    1,
                                                                )
                                                            }
                                                        }
                                                        offset = next_offset;
                                                    }};
                                                }
                                                match option.value {
                                                    IfrTypeValueEnum::U8(u8) => copy_option!(u8),
                                                    IfrTypeValueEnum::U16(u16) => copy_option!(u16),
                                                    IfrTypeValueEnum::U32(u32) => copy_option!(u32),
                                                    IfrTypeValueEnum::U64(u64) => copy_option!(u64),
                                                    _ => (),
                                                }
                                            }
                                            if offset < buffer.len() {
                                                for i in offset..buffer.len() {
                                                    buffer[i] = 0;
                                                }
                                            }
                                        }
                                    } else {
                                        let (kind, value) = unsafe { element.value.to_union() };
                                        user_input.InputValue.Kind = kind;
                                        user_input.InputValue.Value = value;
                                    }
                                    editing = false;
                                }
                                break 'render;
                            }
                        }
                    }
                    Key::Escape => {
                        if editing {
                            editing = false;
                            break 'display;
                        } else if form.FormId != FRONT_PAGE_FORM_ID {
                            user_input.Action = BROWSER_ACTION_FORM_EXIT;
                            break 'render;
                        }
                    }
                    Key::Down => {
                        if editing {
                            if let Some(element) = elements.get_mut(selected) {
                                if element.list {
                                    if element.list_i + 1 < element.options.len() {
                                        element.list_i += 1;
                                    } else {
                                        element.list_i = 0;
                                    }
                                } else {
                                    let i_opt = element
                                        .options
                                        .iter()
                                        .position(|o| o.value == element.value);
                                    if let Some(mut i) = i_opt {
                                        if i + 1 < element.options.len() {
                                            i += 1;
                                        } else {
                                            i = 0;
                                        }
                                        element.value = element.options[i].value;
                                    }
                                }
                            }
                        } else if selected != !0 {
                            let start = selected;
                            loop {
                                if selected + 1 < elements.len() {
                                    selected += 1;
                                } else {
                                    selected = 0;
                                }
                                if let Some(element) = elements.get(selected) {
                                    if element.selectable {
                                        break;
                                    }
                                }
                                if selected == start {
                                    break;
                                }
                            }
                            if selected == 0 {
                                // Handle wrapping
                                element_start = 0;
                            } else if selected - element_start >= max_form_elements {
                                element_start = selected - max_form_elements + 1;
                            }
                        }
                    }
                    Key::Up => {
                        if editing {
                            if let Some(element) = elements.get_mut(selected) {
                                if element.list {
                                    if element.list_i > 0 {
                                        element.list_i -= 1;
                                    } else if !element.options.is_empty() {
                                        element.list_i = element.options.len() - 1;
                                    }
                                } else {
                                    let i_opt = element
                                        .options
                                        .iter()
                                        .position(|o| o.value == element.value);
                                    if let Some(mut i) = i_opt {
                                        if i > 0 {
                                            i -= 1;
                                        } else {
                                            i = element.options.len() - 1;
                                        }
                                        element.value = element.options[i].value;
                                    }
                                }
                            }
                        } else if selected != !0 {
                            let start = selected;
                            loop {
                                if selected > 0 {
                                    selected -= 1;
                                } else {
                                    selected = cmp::max(elements.len(), 1) - 1;
                                }
                                if let Some(element) = elements.get(selected) {
                                    if element.selectable {
                                        break;
                                    }
                                }
                                if selected == start {
                                    break;
                                }
                            }
                            if selected <= element_start {
                                element_start = selected;
                            } else if selected == elements.len() - 1 {
                                // Handle wrapping
                                element_start = if selected >= max_form_elements {
                                    selected - max_form_elements + 1
                                } else {
                                    0
                                };
                            }
                        }
                    }
                    Key::PageDown => {
                        if editing {
                            if let Some(element) = elements.get_mut(selected) {
                                if element.list && element.list_i + 1 < element.options.len() {
                                    element.options.swap(element.list_i, element.list_i + 1);
                                    element.list_i += 1;
                                }
                            }
                        }
                    }
                    Key::PageUp => {
                        if editing {
                            if let Some(element) = elements.get_mut(selected) {
                                if element.list && element.list_i > 0 {
                                    element.list_i -= 1;
                                    element.options.swap(element.list_i, element.list_i + 1);
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    Ok(())
}

extern "efiapi" fn form_display(form: &Form, user_input: &mut UserInput) -> Status {
    form_display_inner(form, user_input).into()
}

extern "efiapi" fn exit_display() {}

extern "efiapi" fn confirm_data_change() -> usize {
    0
}

impl Fde {
    pub fn install() -> Result<()> {
        let guid = guid!("9bbe29e9-fda1-41ec-ad52-452213742d2e");

        let uefi = unsafe { std::system_table_mut() };

        let current = unsafe {
            let mut interface = 0;
            Result::from((uefi.BootServices.LocateProtocol)(&guid, 0, &mut interface))?;
            &mut *(interface as *mut Fde)
        };

        current.FormDisplay = form_display;
        current.ExitDisplay = exit_display;
        current.ConfirmDataChange = confirm_data_change;

        Ok(())
    }
}
