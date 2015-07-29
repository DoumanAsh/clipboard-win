//!Provides direct wrappers to WinAPI functions.
//!
//!These functions omit calls to ```OpenClipboard``` and ```CloseClipboard```.
//!Due to that most functions have requirements for them to be called.

extern crate user32;
extern crate kernel32;

//WinAPI
//types
use winapi::minwindef::{HGLOBAL, UINT};
use winapi::wchar_t; //u16
use winapi::winnt::HANDLE;
use winapi::basetsd::SIZE_T;
//functions
use kernel32::{GlobalFree, GlobalAlloc, GlobalLock, GlobalUnlock, GetLastError};
use user32::{GetClipboardFormatNameW, EnumClipboardFormats, GetClipboardSequenceNumber, SetClipboardData, EmptyClipboard, OpenClipboard, GetClipboardData, CloseClipboard};

//std
use std;
use std::os::windows::ffi::OsStrExt;

//clipboard_win
use super::{WinResult, rust_strlen};

#[inline]
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
            return WinResult(GetLastError());
        }
    }

    WinResult(0)
}

#[inline]
///Wrapper around ```CloseClipboard```.
///
///This function MUST be called prior to successful call of ```open_clipboard```.
pub fn close_clipboard() -> WinResult {
    unsafe {
        if CloseClipboard() == 0 {
            return WinResult(GetLastError());
        }
    }

    WinResult(0)
}

#[inline]
///Wrapper around ```EmptyClipboard```.
///
///This function MUST be called prior to successful call of ```open_clipboard```.
pub fn empty_clipboard() -> WinResult {
    unsafe {
        if EmptyClipboard() == 0 {
            return WinResult(GetLastError());
        }
    }

    WinResult(0)
}

///Wrapper around ```SetClipboardData```.
///
///This function MUST be called prior to successful call of ```open_clipboard```.
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
            return WinResult(GetLastError());
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
            if SetClipboardData(format, handler).is_null() {
                let result = WinResult(GetLastError());
                GlobalFree(handler);
                return result;
            }
        }
    }
    WinResult(0)
}

#[inline(always)]
///Wrapper around ```GetClipboardData``` with hardcoded UTF16 format.
///
///This function MUST be called prior to successful call of ```open_clipboard```.
///
///# Return result:
///
///* ```Ok``` Content of clipboard which is stored in ```String```.
///* ```Err``` Contains ```WinResult```.
pub fn get_clipboard_string() -> Result<String, WinResult> {
    get_clipboard(13)
}

///Wrapper around ```GetClipboardData```.
///
///This function MUST be called prior to successful call of ```open_clipboard```.
///
///# Parameters:
///
///* ```format``` clipboard format code.
///
///# Return result:
///
///* ```Ok``` Content of clipboard which is stored in ```String```.
///* ```Err``` Contains ```WinResult```.
pub fn get_clipboard(format: u32) -> Result<String, WinResult> {
    let result: Result<String, WinResult>;
    unsafe {
        let text_handler: HANDLE = GetClipboardData(format as UINT);
        if text_handler.is_null() {
            result = Err(WinResult(GetLastError()));
        }
        else {
            let text_p = GlobalLock(text_handler) as *const wchar_t;
            let len: usize = rust_strlen(text_p);
            let text_s = std::slice::from_raw_parts(text_p, len);

            result = Ok(String::from_utf16_lossy(text_s));
            GlobalUnlock(text_handler);
        }
    }
    result
}
///Extracts available clipboard formats.
///
///# Return result:
///
///* ```Ok``` Vector of available formats.
///* ```Err``` Contains ```WinResult```.
pub fn get_clipboard_formats() -> Result<Vec<u32>, WinResult> {
    let mut result: Vec<u32> = vec![];
    unsafe {
        let mut clip_format: u32 = EnumClipboardFormats(0);
        while clip_format != 0 {
            result.push(clip_format);
            clip_format = EnumClipboardFormats(clip_format);
        }

        //Error check
        let error = GetLastError();

        if error != 0 {
            return Err(WinResult(error));
        }
    }

    Ok(result)
}

///Returns format name based on it's code.
///
///# Note:
///It is not possible to retrieve name of predefined clipboard format.
///
///# Parameters:
///
///* ```format``` clipboard format code.
///
///# Return result:
///
///* ```Some``` String which contains the format's name.
///* ```None``` If format name is incorrect or predefined.
pub fn get_format_name(format: u32) -> Option<String> {
    let format_buff: [u16; 30] = [0; 30];
    unsafe {
        let buff_p = format_buff.as_ptr() as *mut u16;

        if GetClipboardFormatNameW(format, buff_p, 30) == 0 {
            return None;
        }

    }

    Some(String::from_utf16_lossy(&format_buff))
}
