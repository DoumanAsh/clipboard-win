//!Provides direct wrappers to WinAPI functions.
//!
//!These functions omit calls to ```OpenClipboard``` and ```CloseClipboard``` to be more like
//!wrappers. Due to that it is important that these function will be called upon need.

extern crate user32;
extern crate kernel32;

//WinAPI
//types
use winapi::minwindef::{HGLOBAL, UINT};
use winapi::wchar_t; //u16
use winapi::winnt::HANDLE;
use winapi::basetsd::SIZE_T;
//functions
use user32::{GetClipboardData, SetClipboardData, GetClipboardSequenceNumber, OpenClipboard, CloseClipboard, EmptyClipboard};
use kernel32::{GlobalAlloc, GlobalLock, GlobalUnlock, GetLastError};

//std
use std;

use std::os::windows::ffi::OsStrExt;

//clipboard_win
use super::{WinResult, rust_strlen};

///Wrapper around ```GetClipboardSequenceNumber```.
///
///# Return result:
///
///* ```Some``` Contains return value of ```GetClipboardSequenceNumber```.
///* ```None``` In case if you do not have access. It means that zero is returned by system.
pub fn get_clipboard_seq_num() -> Option<u32> {
    let result: u32 = unsafe { GetClipboardSequenceNumber() };
    if result == 0 { return None; }

    Some(result)
}

#[inline]
///Wrapper around ```OpenClipboard```.
///
///This function MUST be called only once until clipboard will not be closed again with
///```close_clipboard```.
pub fn open_clipboard() -> WinResult {
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return WinResult::new(GetLastError());
        }
    }

    WinResult::new(0)
}

#[inline]
///Wrapper around ```CloseClipboard```.
///
///This function MUST be called only after ```open_clipboard``` has been called.
pub fn close_clipboard() -> WinResult {
    unsafe {
        if CloseClipboard() == 0 {
            return WinResult::new(GetLastError());
        }
    }

    WinResult::new(0)
}

#[inline]
///Wrapper around ```EmptyClipboard```.
///
///This function MUST be called prior to succesful call of ```open_clipboard```.
pub fn empty_clipboard() -> WinResult {
    unsafe {
        if EmptyClipboard() == 0 {
            return WinResult::new(GetLastError());
        }
    }

    WinResult::new(0)
}

///Wrapper around ```SetClipboardData```.
///
///This function MUST be called prior to succesful call of ```open_clipboard```.
pub fn set_clipboard<T: ?Sized + AsRef<std::ffi::OsStr>>(text: &T) -> WinResult {
    let format: UINT = 13; //unicode
    let ghnd: UINT = 66;
    let text = text.as_ref();
    unsafe {
        //allocate buffer and copy string to it.
        let utf16_buff: Vec<u16> = text.encode_wide().collect();
        let len: usize = (utf16_buff.len()+1) * 2;
        let handler: HGLOBAL = GlobalAlloc(ghnd, len as SIZE_T);
        if handler.is_null() {
            return WinResult::new(GetLastError());
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
        }
    }
    WinResult::new(0)
}


///Wrapper around ```GetClipboardData```.
///
///This function MUST be called prior to succesful call of ```open_clipboard```.
pub fn get_clipboard() -> Result<String, String> {
    let cf_unicodetext: UINT = 13;
    let result: Result<String, String>;
    unsafe {
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
    }
    result
}
