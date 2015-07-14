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
//!     set_clipboard("for my waifu!").unwrap();
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
pub struct WinResult {
    errno: u32,
}

impl WinResult {
    ///Constructs new error.
    pub fn new(errno: u32) -> WinResult {
        WinResult {
            errno: errno,
        }
    }

    #[inline(always)]
    ///Returns ```true``` if result is ok
    pub fn is_ok(&self) -> bool {
        self.errno == 0
    }

    #[inline(always)]
    ///Returns ```true``` if result is not ok
    pub fn is_not_ok(&self) -> bool {
        !self.is_ok()
    }

    #[inline(always)]
    ///Returns extended error code. Should be used in case if result is not ok.
    pub fn errno(&self) -> u32 {
        self.errno
    }

    ///Returns description of WinAPI error code.
    pub fn errno_desc(&self) -> String {
        let mut format_buff: [u16; 300] = [0; 300];
        let num_chars: u32 = unsafe { FormatMessageW(0x00000200 | 0x00001000 | 0x00002000,
                                                     std::ptr::null(), self.errno,
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
        self.errno == right.errno
    }

    fn ne(&self, right: &WinResult) -> bool {
        self.errno != right.errno
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
    err_fn: fn(&String) -> (),
}

impl ClipboardManager {
    fn default_ok(text: &String) -> () { println!("Clipboard content: {}", &text); }
    fn default_err(err_text: &String) -> () { println!("Failed to get clipboard. Reason:{}", &err_text); }
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
    ///Error description is passed from ```get_clipboard()```
    pub fn err_callback(&mut self, callback: fn(&String) -> ()) -> &mut ClipboardManager
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
                match get_clipboard() {
                    Ok(clip_text) => { (self.ok_fn)(&clip_text) },
                    Err(err_text) => { (self.err_fn)(&err_text) },
                }
            println!(">>>");
            }
            std::thread::sleep_ms(self.delay_ms);
        }
    }
}

///Set clipboard with text.
///
///# Return result:
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
///# Return result:
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

///Extracts available clipboard formats.
///
///# Return result:
///
///* ```Ok``` Vector of available formats.
///* ```Err``` Error description.
pub fn get_clipboard_formats() -> Result<Vec<u32>, u32> {
    let mut result: Vec<u32> = vec![];
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err(GetLastError());
        }

        let mut clip_format: u32 = EnumClipboardFormats(0);
        while clip_format != 0 {
            result.push(clip_format);
            clip_format = EnumClipboardFormats(clip_format);
        }

        //Error check
        let error = GetLastError();

        if error != 0 {
            return Err(error);
        }

        CloseClipboard();
    }

    Ok(result)
}

///Returns format name based on it's code.
///
///# Note:
///It is not possible to retrieve clipboard format name of predefined one.
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

    if let Ok(format_name) = String::from_utf16(&format_buff) {
        return Some(format_name);
    }
    None
}
