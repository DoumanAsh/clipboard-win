//!Provides direct wrappers to WinAPI functions.

//WinAPI
extern crate user32;
extern crate kernel32;

use user32::{GetClipboardSequenceNumber, OpenClipboard, CloseClipboard, EmptyClipboard};
use kernel32::{GetLastError};

//std
use std;

//clipboard_win
use super::WinResult;

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
pub fn empty_clipboard() -> WinResult {
    unsafe {
        if EmptyClipboard() == 0 {
            return WinResult::new(GetLastError());
        }
    }

    WinResult::new(0)
}
