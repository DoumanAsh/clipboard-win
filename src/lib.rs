//! Clipboard WIN API
//!
//! This crate provide simple means to operate with Windows clipboard.
//!
//! # Example:
//! ```
//! extern crate clipboard_win;
//!
//! use clipboard_win::*;
//!
//! fn main() {
//!     println!("I set some clipboard text like a boss!");
//!     set_clipboard("for my waifu!").unwrap();
//! }
//! ```

extern crate winapi;
extern crate user32;
extern crate kernel32;

//rust
use std::os::windows::ffi::OsStrExt;

//WINAPI
//types
use winapi::minwindef::{HGLOBAL, UINT};
use winapi::wchar_t; //u16
use winapi::winnt::HANDLE;
use winapi::basetsd::SIZE_T;
//functions
use kernel32::{GlobalAlloc, GlobalLock, GlobalUnlock, GetLastError};
use user32::{GetClipboardSequenceNumber, SetClipboardData, EmptyClipboard, OpenClipboard, GetClipboardData, CloseClipboard};

///GetClipboardSequenceNumber wrapper.
///
///Return result:
///
///* ```Some``` Upon successful retrieval of sequence number.
///* ```None``` In case if you do not have access. It means that zero is returned by system.
pub fn get_clipboard_seq_num() -> Option<u32> {
    let result: u32 = unsafe { GetClipboardSequenceNumber() };
    if result == 0 { return None; }

    Some(result)
}

///Set clipboard with text.
///
///Return result:
///
///* ```Ok``` Upon succesful set of text.
///* ```Err``` Otherwise. See [Error codes](https://msdn.microsoft.com/en-us/library/windows/desktop/ms681381%28v=vs.85%29.aspx)
pub fn set_clipboard<T: ?Sized + AsRef<std::ffi::OsStr>>(text: &T) -> Result<(), u32> {
    let format: UINT = 13; //unicode
    let ghnd: UINT = 66;
    let text = text.as_ref();
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err(GetLastError());
        }

        //allocate buffer and copy string to it.
        let utf16_buff: Vec<u16> = text.encode_wide().collect();
        let len: usize = (utf16_buff.len()+1) * 2;
        let handler: HGLOBAL = GlobalAlloc(ghnd, len as SIZE_T);
        if handler.is_null() {
            CloseClipboard();
            return Err(GetLastError());
        }
        else {
            let lock = GlobalLock(handler) as *mut u16;

            let len: usize = (len - 1) / 2;
                                          //src,         dest, len
            std::ptr::copy_nonoverlapping(utf16_buff.as_ptr(), lock, len);
            let len: isize = len as isize;
            *lock.offset(len) = 0;

            GlobalUnlock(handler);

            //Set new clipboard text.
            EmptyClipboard();
            SetClipboardData(format, handler);
            CloseClipboard();
        }
    }
    Ok(())
}

///Rust variant of strlen.
///
///* ```buff_p``` Must be valid non-NULL pointer.
#[inline(always)]
pub unsafe fn rust_strlen(buff_p: *const u16) -> usize {
    let mut idx: isize = 0;
    while *buff_p.offset(idx) != 0 { idx += 1; }
    idx as usize
}

///Extracts clipboard content and convert it to String.
///
///Return result:
///
///* ```Ok``` Content of clipboard which is stored in ```String```.
///* ```Err``` Error description.
pub fn get_clipboard() -> Result<String, String> {
    let cf_unicodetext: UINT = 13;
    let result: Result<String, String>;
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            //Leave earlier as clipboard is closed at the end
            result = Err(format!("Unable to open clipboard. Errno:{}", GetLastError()));
        }
        else {
            let text_handler: HANDLE = GetClipboardData(cf_unicodetext);
            if text_handler.is_null() {
                result = Err(format!("Unable to get clipboard. Errno:{}", GetLastError()));
            }
            else {
                let text_p = GlobalLock(text_handler) as *const wchar_t;
                let len: usize = rust_strlen(text_p);
                let text_s = std::slice::from_raw_parts(text_p, len);

                result = String::from_utf16(text_s).map_err(| err | format!("Failed to parse clipboard's text. Errno:{:?}", err));
                GlobalUnlock(text_handler);
            }
            CloseClipboard();
        }
    }
    result
}
