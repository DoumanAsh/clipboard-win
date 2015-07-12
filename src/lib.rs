/// Clipboard WIN API

extern crate winapi;
extern crate user32;
extern crate kernel32;
extern crate num;

//rust

//num
use num::zero;
//WINAPI
//types
use winapi::minwindef::{HGLOBAL, UINT};
use winapi::wchar_t; //u16
use winapi::winnt::HANDLE;
use winapi::basetsd::SIZE_T;
//functions
use kernel32::{GlobalAlloc, GlobalLock, GlobalUnlock};
use user32::{SetClipboardData, EmptyClipboard, OpenClipboard, GetClipboardData, CloseClipboard};

///Set clipboard with text.
pub fn set_clipboard(text: &str) {
    let format: UINT = 13; //unicode
    let ghnd: UINT = 0x0042;
    let len: usize = text.len() * 2;
    unsafe {
        //allocate buffer and copy string to it.
        let handler: HGLOBAL = GlobalAlloc(ghnd, len as SIZE_T);
        let lock = GlobalLock(handler) as *mut u8;

                                      //src,         dest, len
        std::ptr::copy_nonoverlapping(text.as_ptr(), lock, len);
        std::ptr::write_bytes(lock.offset(len as isize), 0, 2);

        GlobalUnlock(handler);

        //Set new clipboard text.
        EmptyClipboard();
        OpenClipboard(std::ptr::null_mut());
        SetClipboardData(format, handler);
        CloseClipboard();
    }
}

///Rust variant of strlen.
///
///@buff_p Must be valid non-NULL pointer.
#[inline(always)]
pub unsafe fn rust_strlen<T: PartialEq + num::traits::Zero>(buff_p: *const T) -> usize {
    let mut idx: isize = 0;
    while *buff_p.offset(idx) != zero() { idx += 1; }
    idx as usize
}

///Extracts clipboard content and convert it to String.
pub fn get_clipboard() -> Result<String, std::string::FromUtf16Error> {
    let cf_unicodetext: UINT = 13;
    let result: Result<String, std::string::FromUtf16Error>;
    unsafe {
        OpenClipboard(std::ptr::null_mut());

        let text_handler: HANDLE = GetClipboardData(cf_unicodetext);
        let text_p = GlobalLock(text_handler) as *const wchar_t;
        let len: usize = rust_strlen(text_p);
        let text_s = std::slice::from_raw_parts(text_p, len);

        result = String::from_utf16(text_s);
        GlobalUnlock(text_handler);
        CloseClipboard();
    }
    result
}
