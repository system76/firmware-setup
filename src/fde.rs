use orbclient::{Color, Renderer};
use orbfont::{Font, Text};
use std::{char, cmp, mem, ptr};
use std::ops::Try;
use std::proto::Protocol;
use uefi::{Event, Handle};
use uefi::boot::InterfaceType;
use uefi::guid::Guid;
use uefi::hii::{AnimationId, FormId, ImageId, QuestionId, StringId};
use uefi::hii::database::HiiHandle;
use uefi::hii::ifr::{
    HiiDate, HiiRef, HiiTime, HiiValue,
    IfrOpCode, IfrOpHeader, IfrTypeKind, IfrTypeValue, IfrTypeValueEnum,
    IfrAction, IfrCheckbox, IfrNumeric, IfrOneOf, IfrOneOfOption, IfrRef, IfrSubtitle
};
use uefi::status::{Error, Result, Status};
use uefi::text::TextInputKey;

use crate::display::{Display, Output, ScaledDisplay};
use crate::key::{key, Key};

// TODO: Move to uefi library {
pub const HII_STRING_PROTOCOL_GUID: Guid = Guid(0xfd96974, 0x23aa, 0x4cdc, [0xb9, 0xcb, 0x98, 0xd1, 0x77, 0x50, 0x32, 0x2a]);

#[repr(C)]
pub struct HiiStringProtocol {
    pub NewString: extern "win64" fn(), //TODO
    pub GetString: extern "win64" fn(
        &HiiStringProtocol,
        Language: *const u8,
        PackageList: HiiHandle,
        StringId: StringId,
        String: *mut u16,
        StringSize: &mut usize,
        StringFontInfo: usize, // TODO
    ) -> Status,
    pub SetString: extern "win64" fn(), //TODO
    pub GetLanguages: extern "win64" fn(), //TODO
    pub GetSecondaryLanguages: extern "win64" fn(), //TODO
}

impl HiiStringProtocol {
    pub fn string(&self, PackageList: HiiHandle, StringId: StringId) -> Result<String> {
        let mut data = vec![0u16; 4096];
        let mut len = data.len();
        (self.GetString)(
            self,
            b"en-US\0".as_ptr(),
            PackageList,
            StringId,
            data.as_mut_ptr(),
            &mut len,
            0
        )?;
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
        &*((addr - offset) as *const T)
    }

    unsafe fn object_at_mut(&mut self, offset: usize) -> &mut T {
        let addr = self as *mut Self as usize;
        &mut *((addr - offset) as *mut T)
    }
}

pub trait ListEntryObject<T> {
    unsafe fn object(&self) -> &T;

    unsafe fn object_mut(&mut self) -> &mut T;
}

macro_rules! list_entry {
    ($t:ty, $l:tt) => (
        impl ListEntryObject<$t> for ListEntry<$t> {
            unsafe fn object(&self) -> &$t {
                self.object_at(offset_of!($t, $l))
            }

            unsafe fn object_mut(&mut self) -> &mut $t {
                self.object_at_mut(offset_of!($t, $l))
            }
        }
    );
}

pub struct ListEntryIter<'a, T> {
    start: Option<&'a ListEntry<T>>,
    current: Option<&'a ListEntry<T>>,
}

impl<'a, T> Iterator for ListEntryIter<'a, T> where ListEntry<T>: ListEntryObject<T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;
        let next = current.next();
        if next.map(|x| x as *const _) == self.start.map(|x| x as *const _) {
            self.current = None;
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

pub type ValidateQuestion = extern "win64" fn (
    Form: &Form,
    Statement: &Statement,
    Value: &HiiValue,
    ErrorInfo: &mut StatementErrorInfo,
) -> u32;

pub type PasswordCheck = extern "win64" fn(
    Form: &Form,
    Statement: &Statement,
    PasswordString: *const u16
) -> Status;

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
    pub HighlightedStatement: *const Statement,
    pub FormRefreshEvent: Event,
    pub HotKeyListHead: ListHead<HotKey>,
    pub ImageId: ImageId,
    pub AnimationId: AnimationId,
    pub BrowserStatus: u32,
    pub ErrorString: *const u16,
}

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
    pub FormDisplay: extern "win64" fn(FormData: &Form, UserInputData: &mut UserInput) -> Status,
    pub ExitDisplay: extern "win64" fn(),
    pub ConfirmDataChange: extern "win64" fn() -> usize,
}

static mut DISPLAY: *mut Display = ptr::null_mut();
static mut FONT: *const Font = ptr::null_mut();

struct ElementOption {
    option_ptr: *const QuestionOption,
    prompt: String,
    value: IfrTypeValueEnum,
}

struct Element {
    statement_ptr: *const Statement,
    prompt: String,
    value: IfrTypeValueEnum,
    options: Vec<ElementOption>,
    selectable: bool,
    editable: bool,
}

fn form_display_inner(form: &Form, user_input: &mut UserInput) -> Result<()> {
    debugln!();
    debugln!("form_display");
    debugln!("FORM_DISPLAY_ENGINE_FORM {}", mem::size_of_val(form));
    debugln!("EFI_HII_VALUE {}",  mem::size_of_val(user_input));
    debugln!("HII_VALUE {}", mem::size_of::<HiiValue>());
    debugln!("EFI_IFR_TYPE_VALUE {}", mem::size_of::<IfrTypeValue>());
    debugln!("EFI_GUID {}", mem::size_of::<Guid>());
    debugln!("EFI_HII_TIME {}", mem::size_of::<HiiTime>());
    debugln!("EFI_HII_DATE {}", mem::size_of::<HiiDate>());
    debugln!("EFI_HII_REF {}", mem::size_of::<HiiRef>());
    debugln!("EFI_IFR_CHECKBOX {}", mem::size_of::<IfrCheckbox>());
    debugln!("EFI_IFR_SUBTITLE {}", mem::size_of::<IfrSubtitle>());

    let hii_string = <&'static mut HiiStringProtocol>::one()?;

    let string = |string_id: StringId| -> Result<String> {
         hii_string.string(form.HiiHandle, string_id)
    };

    let mut selected = !0;
    let mut editing = false;
    let mut elements = Vec::new();
    for statement in form.StatementListHead.iter() {
        let statement_ptr = statement as *const _;
        debugln!("statement: {:p}", statement_ptr);

        let mut options = Vec::new();
        for option in statement.OptionListHead.iter() {
            let option_ptr = option as *const _;
            debugln!("  option: {:p}", option_ptr);
            if let Some(op) = option.OptionOpCode() {
                let value = unsafe {
                    op.Value.to_enum(op.Kind)
                };
                debugln!("    {:?}: {:?}", op.Option, value);
                if let Ok(prompt) = string(op.Option) {
                    options.push(ElementOption {
                        option_ptr,
                        prompt,
                        value,
                    });
                }
            }
        }

        let mut add_element = |string_id: StringId, selectable: bool, editable: bool| {
            let value = unsafe {
                statement.CurrentValue.Value.to_enum(statement.CurrentValue.Kind)
            };
            debugln!("    {:?}: {:?}", string_id, value);
            if let Ok(prompt) = string(string_id) {
                if statement_ptr == form.HighlightedStatement || (selected == !0 && selectable) {
                    selected = elements.len();
                }
                elements.push(Element {
                    statement_ptr,
                    prompt,
                    options,
                    selectable,
                    editable,
                    value,
                });
            }
        };

        if let Some(op) = statement.OpCode() {
            debugln!("  {:?}", op);
            match op.OpCode {
                IfrOpCode::Action => if let Some(action) = unsafe { op.cast::<IfrAction>() } {
                    add_element(action.QuestionHeader.Header.Prompt, true, false);
                },
                IfrOpCode::Checkbox => if let Some(checkbox) = unsafe { op.cast::<IfrCheckbox>() } {
                    add_element(checkbox.Question.Header.Prompt, true, true);
                },
                IfrOpCode::Numeric => if let Some(numeric) = unsafe { op.cast::<IfrNumeric>() } {
                    add_element(numeric.Question.Header.Prompt, true, true);
                },
                IfrOpCode::OneOf => if let Some(one_of) = unsafe { op.cast::<IfrOneOf>() } {
                    add_element(one_of.Question.Header.Prompt, true, true);
                },
                IfrOpCode::Ref => if let Some(ref_) = unsafe { op.cast::<IfrRef>() } {
                    add_element(ref_.Question.Header.Prompt, true, false);
                },
                IfrOpCode::Subtitle => if let Some(subtitle) = unsafe { op.cast::<IfrSubtitle>() } {
                    add_element(subtitle.Statement.Prompt, false, false);
                },
                _ => ()
            }
        }
    }

    let mut display = unsafe {
        if DISPLAY.is_null() {
            let display = Display::new(Output::one()?);
            DISPLAY = Box::into_raw(Box::new(display));
        }
        ScaledDisplay::new(&mut *DISPLAY)
    };

    let font = unsafe {
        if FONT.is_null() {
            let font = match Font::from_data(crate::app::FONTTTF) {
                Ok(ok) => ok,
                Err(err) => {
                    println!("failed to parse font: {}", err);
                    return Err(Error::NotFound);
                }
            };
            FONT = Box::into_raw(Box::new(font));
        }
        &*FONT
    };

    let title_opt = string(form.FormTitle).ok();
    'display: loop {
        let (display_w, display_h) = (display.width(), display.height());

        display.clear();

        let font_size = (display_h as f32) / 26.0;

        let black = Color::rgb(0x00, 0x00, 0x00);
        let white = Color::rgb(0xFF, 0xFF, 0xFF);

        let mut draw_rendered = |x: i32, y: i32, rendered: &Text, highlighted: bool| {
            let (fg, bg) = if highlighted {
                (black, white)
            } else {
                (white, black)
            };
            display.rect(x, y, rendered.width(), rendered.height(), bg);
            rendered.draw(&mut display, x, y, fg);
        };

        let mut y = 0;

        if editing {
            if let Some(element) = elements.get(selected) {
                {
                    // TODO: Do not render in drawing loop
                    let rendered = font.render(&element.prompt, font_size);
                    let x = (display_w as i32 - rendered.width() as i32) / 2;
                    draw_rendered(x, y, &rendered, false);
                    y += rendered.height() as i32;
                }

                for option in element.options.iter() {
                    let h = {
                        // TODO: Do not render in drawing loop
                        let rendered = font.render(&option.prompt, font_size);
                        draw_rendered(16, y, &rendered, option.value == element.value);
                        rendered.height() as i32
                    };

                    y += h;
                }
            } else {
                editing = false;
                continue 'display;
            }
        } else {
            if let Some(ref title) = title_opt {
                // TODO: Do not render in drawing loop
                let rendered = font.render(&title, font_size);
                let x = (display_w as i32 - rendered.width() as i32) / 2;
                draw_rendered(x, y, &rendered, false);
                y += rendered.height() as i32;
            }

            for (i, element) in elements.iter().enumerate() {
                let h = {
                    // TODO: Do not render in drawing loop
                    let rendered = font.render(&element.prompt, font_size);
                    draw_rendered(16, y, &rendered, i == selected);
                    rendered.height() as i32
                };

                if let Some(option) = element.options.iter().find(|o| o.value == element.value) {
                    // TODO: Do not render in drawing loop
                    let rendered = font.render(&option.prompt, font_size);
                    draw_rendered(display_w as i32 / 2, y, &rendered, false);
                } else if element.editable {
                    // TODO: Do not render in drawing loop
                    let rendered = font.render(&format!("{:?}", element.value), font_size);
                    draw_rendered(display_w as i32 / 2, y, &rendered, false);
                }

                y += h;
            }
        }

        display.sync();

        match key()? {
            Key::Enter => {
                debugln!("enter");
                if let Some(element) = elements.get(selected) {
                    if element.editable && ! editing {
                        editing = true;
                    } else {
                        user_input.SelectedStatement = element.statement_ptr;
                        if editing {
                            let (kind, value) = unsafe { element.value.to_union() };
                            user_input.InputValue.Kind = kind;
                            user_input.InputValue.Value = value;
                            editing = false;
                        } else {
                            unsafe {
                                ptr::copy(
                                    &(*element.statement_ptr).CurrentValue,
                                    &mut user_input.InputValue,
                                    1
                                );
                            }
                        }
                        break 'display;
                    }
                }
            },
            Key::Escape => {
                debugln!("escape");
                if editing {
                    editing = false;
                } else {
                    user_input.Action = (1 << 17);
                    break 'display;
                }
            },
            Key::Down => {
                debugln!("down");
                if editing {
                    if let Some(mut element) = elements.get_mut(selected) {
                        let i_opt = element.options.iter().position(|o| o.value == element.value);
                        if let Some(mut i) = i_opt {
                            if i + 1 < element.options.len() {
                                i += 1;
                            } else {
                                i = 0;
                            }
                            element.value = element.options[i].value;
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
                }
            },
            Key::Up => {
                debugln!("up");
                if editing {
                    if let Some(mut element) = elements.get_mut(selected) {
                        let i_opt = element.options.iter().position(|o| o.value == element.value);
                        if let Some(mut i) = i_opt {
                            if i > 0 {
                                i -= 1;
                            } else {
                                i = element.options.len() - 1;
                            }
                            element.value = element.options[i].value;
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
                }
            },
            other => {
                debugln!("{:?}", other);
            },
        }
    }

    debugln!("selected: {:p}, action: {:#x}", user_input.SelectedStatement, user_input.Action);

    Ok(())
}

extern "win64" fn form_display(form: &Form, user_input: &mut UserInput) -> Status {
    match form_display_inner(form, user_input) {
        Ok(ok) => Status::from_ok(0),
        Err(err) => Status::from_error(err),
    }
}

extern "win64" fn exit_display() {
    debugln!("exit_display");
}

extern "win64" fn confirm_data_change() -> usize {
    debugln!("confirm_data_change");
    0
}

impl Fde {
    pub fn new() -> Fde {
        Fde {
            FormDisplay: form_display,
            ExitDisplay: exit_display,
            ConfirmDataChange: confirm_data_change,
        }
    }

    pub fn install(&mut self) -> Result<()> {
        let guid = Guid(0x9bbe29e9, 0xfda1, 0x41ec, [0xad, 0x52, 0x45, 0x22, 0x13, 0x74, 0x2d, 0x2e]);

        let uefi = unsafe { std::system_table_mut() };

        let current = unsafe {
            let mut interface = 0;
            (uefi.BootServices.LocateProtocol)(&guid, 0, &mut interface)?;
            &mut *(interface as *mut Fde)
        };

        debugln!("Current FDE: {:#p}", current);

        unsafe { ptr::write(current, Fde::new()); }

        Ok(())
    }
}
