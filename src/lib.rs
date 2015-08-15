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

//WinAPI
//functions
use winapi::{DWORD};
use kernel32::{FormatMessageW};

//wrapper functions
pub mod wrapper;
use wrapper::{get_clipboard_seq_num};

mod constants;
use constants::*;

use std::error::Error;
use std::fmt;

#[derive(Clone)]
///Represents Windows error code.
pub struct WindowsError(u32);

impl WindowsError {
    ///Constructs new error.
    pub fn new(errno: u32) -> WindowsError {
        WindowsError(errno)
    }

    #[inline(always)]
    ///Returns underlying error code.
    pub fn errno(&self) -> u32 {
        self.0
    }

    ///Returns description of underlying error code.
    pub fn errno_desc(&self) -> String {
        const BUF_SIZE: usize = 512;
        let mut format_buff: [u16; BUF_SIZE] = [0; BUF_SIZE];
        let fmt_flags: DWORD = FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ARGUMENT_ARRAY;
        let num_chars: u32 = unsafe { FormatMessageW(fmt_flags,
                                                     std::ptr::null(), self.0,
                                                     0, format_buff.as_mut_ptr(),
                                                     BUF_SIZE as u32, std::ptr::null_mut()) };

        let num_chars: usize = num_chars as usize;
        //Errors are formatted with windows new lines at the end.
        //If string does not end with /r/n then, most possibly, it is not a error
        //but some other system thing(who knows what...)
        if num_chars == 0 || format_buff[num_chars-1] != 10 {
            return "Unknown Error.".to_string();
        }
        String::from_utf16_lossy(&format_buff[0..num_chars-2])
    }
}

//Own debug as derive one is a bit lame
impl fmt::Debug for WindowsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "WinAPI Error({})", self.errno())
    }
}

impl fmt::Display for WindowsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "WinAPI Error({})", self.errno())
    }
}

impl Error for WindowsError {
    fn description(&self) -> &str {
        "WinAPI Error"
    }
}

impl PartialEq for WindowsError {
    fn eq(&self, right: &WindowsError) -> bool {
        self.0 == right.0
    }

    fn ne(&self, right: &WindowsError) -> bool {
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
    err_fn: fn(&WindowsError) -> (),
}

impl ClipboardManager {
    fn default_ok(text: &String) -> () { println!("Clipboard content: {}", &text); }
    fn default_err(err_code: &WindowsError) -> () { println!("Failed to get clipboard. Reason:{}", err_code.errno_desc()); }
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
    pub fn err_callback(&mut self, callback: fn(&WindowsError) -> ()) -> &mut ClipboardManager
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
pub fn set_clipboard<T: ?Sized + AsRef<std::ffi::OsStr>>(text: &T) -> Result<(), WindowsError> {
    try!(wrapper::open_clipboard());
    let result = wrapper::set_clipboard(text);
    try!(wrapper::close_clipboard());
    result
}

#[inline(always)]
///Retrieves clipboard content in UTF16 format and convert it to String.
///
///# Return result:
///
///* ```Ok``` Content of clipboard which is stored in ```String```.
///* ```Err``` Contains ```WindowsError```.
pub fn get_clipboard_string() -> Result<String, WindowsError> {
    try!(wrapper::open_clipboard());
    let result = wrapper::get_clipboard_string();
    try!(wrapper::close_clipboard());
    result
}

///Retrieves clipboard content.
///
///# Parameters:
///
///* ```format``` clipboard format code.
///
///# Return result:
///
///* ```Ok``` Contains buffer with raw data.
///* ```Err``` Contains ```WindowsError```.
pub fn get_clipboard(format: u32) -> Result<Vec<u8>, WindowsError> {
    try!(wrapper::open_clipboard());
    let result = wrapper::get_clipboard(format);
    try!(wrapper::close_clipboard());
    result
}

///Extracts available clipboard formats.
///
///# Return result:
///
///* ```Ok``` Vector of available formats.
///* ```Err``` Error description.
pub fn get_clipboard_formats() -> Result<Vec<u32>, WindowsError> {
    try!(wrapper::open_clipboard());
    let result = wrapper::get_clipboard_formats();
    try!(wrapper::close_clipboard());
    result
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
    if wrapper::open_clipboard().is_err() { return None; }
    let result = wrapper::get_format_name(format);
    if wrapper::close_clipboard().is_err() { return None; }
    result
}
