// SPDX-License-Identifier: GPL-3.0-only

// `LIST_ENTRY` implemenation
// TODO: Move to uefi library

#[repr(C)]
pub struct ListEntry<T> {
    pub Flink: *mut ListEntry<T>,
    pub Blink: *mut ListEntry<T>,
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

    pub unsafe fn object_at(&self, offset: usize) -> &T {
        let addr = self as *const Self as usize;
        unsafe { &*((addr - offset) as *const T) }
    }

    pub unsafe fn object_at_mut(&mut self, offset: usize) -> &mut T {
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
                unsafe { self.object_at(core::mem::offset_of!($t, $l)) }
            }

            unsafe fn object_mut(&mut self) -> &mut $t {
                unsafe { self.object_at_mut(core::mem::offset_of!($t, $l)) }
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
    pub fn iter(&self) -> ListEntryIter<'_, T> {
        let next = self.0.next();
        ListEntryIter {
            start: next,
            current: next,
        }
    }
}
