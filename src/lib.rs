//! Clipboard WinAPI
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
//!     set_clipboard("for my waifu!");
//! }
//! ```

extern crate winapi;
extern crate user32;
extern crate kernel32;

//rust
use std::os::windows::ffi::OsStrExt;

//WinAPI
//types
use winapi::minwindef::{HGLOBAL, UINT};
use winapi::wchar_t; //u16
use winapi::winnt::HANDLE;
use winapi::basetsd::SIZE_T;
//functions
use kernel32::{GlobalAlloc, GlobalLock, GlobalUnlock, GetLastError, FormatMessageW};
use user32::{GetClipboardFormatNameW, EnumClipboardFormats, SetClipboardData, EmptyClipboard, OpenClipboard, GetClipboardData, CloseClipboard};

//wrapper functions
pub mod wrapper;
use wrapper::{get_clipboard_seq_num};

#[derive(Debug, Clone)]
///Represents Windows result.
pub struct WinResult(u32);

impl WinResult {
    ///Custom errors

    ///Constructs new error.
    pub fn new(errno: u32) -> WinResult {
        WinResult(errno)
    }

    #[inline(always)]
    ///Returns ```true``` if result is ok
    pub fn is_ok(&self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    ///Returns ```true``` if result is not ok
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    #[inline(always)]
    ///Returns extended error code. Should be used in case if result is not ok.
    pub fn errno(&self) -> u32 {
        self.0
    }

    ///Returns description of WinAPI error code.
    pub fn errno_desc(&self) -> String {
        let mut format_buff: [u16; 300] = [0; 300];
        let num_chars: u32 = unsafe { FormatMessageW(0x00000200 | 0x00001000 | 0x00002000,
                                                     std::ptr::null(), self.0,
                                                     0, format_buff.as_mut_ptr(),
                                                     200 as u32, std::ptr::null_mut()) };

        if num_chars == 0 {
            return "Unknown error".to_string();
        }

        String::from_utf16(&format_buff).unwrap_or("Unknown error".to_string())
    }
}

impl PartialEq for WinResult {
    fn eq(&self, right: &WinResult) -> bool {
        self.0 == right.0
    }

    fn ne(&self, right: &WinResult) -> bool {
        self.0 != right.0
    }
}

///Clipboard manager provides a primitive hack for console application to handle updates of
///clipboard. It uses ```get_clipboard_seq_num``` to determines whatever clipboard is updated or
///not. Due to that this manager is a bit hacky and not exactly right way to properly work with
///clipboard. On other hand it is the best and most easy option for console application as a proper
///window is required to be created to work with clipboard.
pub struct ClipboardManager {
    delay_ms: u32,
    ok_fn: fn(&String) -> (),
    err_fn: fn(&WinResult) -> (),
}

impl ClipboardManager {
    fn default_ok(text: &String) -> () { println!("Clipboard content: {}", &text); }
    fn default_err(err_code: &WinResult) -> () { println!("Failed to get clipboard. Reason:{}", err_code.errno_desc()); }
    ///Constructs new ClipboardManager with default settings
    pub fn new() -> ClipboardManager {
        ClipboardManager {
            delay_ms: 100,
            ok_fn: ClipboardManager::default_ok,
            err_fn: ClipboardManager::default_err,
        }
    }

    ///Configure manager's delay between accessing clipboard.
    pub fn delay(&mut self, delay_ms: u32) -> &mut ClipboardManager {
        self.delay_ms = delay_ms;
        self
    }

    ///Sets callback for successfully retrieved clipboard's text.
    pub fn ok_callback(&mut self, callback: fn(&String) -> ()) -> &mut ClipboardManager
     {
        self.ok_fn = callback;
        self
    }

    ///Sets callback for failed retrieval of clipboard's text.
    ///
    ///Error code is passed from ```get_clipboard_string()```
    pub fn err_callback(&mut self, callback: fn(&WinResult) -> ()) -> &mut ClipboardManager
     {
        self.err_fn = callback;
        self
    }

    ///Starts manager loop.
    ///
    ///It's infinitely running with delay and checking whatever clipboard is updated.
    ///In case if it is updated callbacks will be called. Depending on whatever clipboard's text
    ///was retrieved or not right callback will be called.
    pub fn run(&self) -> () {
        let mut clip_num: u32 = get_clipboard_seq_num().unwrap_or_else(|| panic!("Lacks sufficient rights to access clipboard(WINSTA_ACCESSCLIPBOARD)"));
        loop {
            // It is very unlikely that we would suddenly start to lack access rights.
            // So let's just skip this iteration. Maybe it is just Windows bug... ^_^
            let new_num = get_clipboard_seq_num().unwrap_or(0);
            if new_num != 0 && clip_num != new_num {
                clip_num = new_num;
                match get_clipboard_string() {
                    Ok(clip_text) => { (self.ok_fn)(&clip_text) },
                    Err(err_code) => { (self.err_fn)(&err_code) },
                }
            println!(">>>");
            }
            std::thread::sleep_ms(self.delay_ms);
        }
    }
}

///Set clipboard with text.
///
pub fn set_clipboard<T: ?Sized + AsRef<std::ffi::OsStr>>(text: &T) -> WinResult {
    let format: UINT = 13; //unicode
    let ghnd: UINT = 66;
    let text = text.as_ref();
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return WinResult(GetLastError());
        }

        //allocate buffer and copy string to it.
        let utf16_buff: Vec<u16> = text.encode_wide().collect();
        let len: usize = (utf16_buff.len()+1) * 2;
        let handler: HGLOBAL = GlobalAlloc(ghnd, len as SIZE_T);
        if handler.is_null() {
            CloseClipboard();
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
            SetClipboardData(format, handler);
            CloseClipboard();
        }
    }
    WinResult(0)
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
///# Return result:
///
///* ```Ok``` Content of clipboard which is stored in ```String```.
///* ```Err``` Contains ```WinResult```.
pub fn get_clipboard_string() -> Result<String, WinResult> {
    let cf_unicodetext: UINT = 13;
    let result: Result<String, WinResult>;
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            //Leave earlier as clipboard is closed at the end
            result = Err(WinResult(GetLastError()));
        }
        else {
            let text_handler: HANDLE = GetClipboardData(cf_unicodetext);
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
            CloseClipboard();
        }
    }
    result
}

///Extracts available clipboard formats.
///
///# Return result:
///
///* ```Ok``` Vector of available formats.
///* ```Err``` Error description.
pub fn get_clipboard_formats() -> Result<Vec<u32>, WinResult> {
    let mut result: Vec<u32> = vec![];
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err(WinResult(GetLastError()));
        }

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

        CloseClipboard();
    }

    Ok(result)
}

///Returns format name based on it's code.
///
///# Note:
///It is not possible to retrieve name of predefined clipboard format.
///
///# Return result:
///
///* ```Some``` String which contains the format's name.
///* ```None``` If format name is incorrect or predefined.
pub fn get_format_name(format: u32) -> Option<String> {
    let format_buff: [u16; 30] = [0; 30];
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return None;
        }

        let buff_p = format_buff.as_ptr() as *mut u16;

        if GetClipboardFormatNameW(format, buff_p, 30) == 0 {
            return None;
        }

        CloseClipboard();
    }

    Some(String::from_utf16_lossy(&format_buff))
}
