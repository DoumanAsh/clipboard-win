//! Clipboard WinAPI
//!
//! This crate provide simple means to operate with Windows clipboard.
//!
//! # Example:
//! ```
//! extern crate clipboard_win;
//!
//! use clipboard_win::set_clipboard;
//!
//! fn main() {
//!     println!("I set some clipboard text like a boss!");
//!     set_clipboard("for my waifu!");
//! }
//! ```

extern crate winapi;
extern crate user32;
extern crate kernel32;

use winapi::{DWORD};
use kernel32::{FormatMessageW};

mod constants;
use constants::{FORMAT_MESSAGE_IGNORE_INSERTS, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_ARGUMENT_ARRAY};
pub mod clipboard_formats;

pub mod wrapper;

use std::error::Error;
use std::fmt;

///try clone that returns None instead of Err<T>
macro_rules! try_none {
    ($expr:expr) => (match $expr {
        Ok(val) => val,
        Err(_) => return None,
    })
}

///Checks format availability.
///
///Returns WinError if it is not available.
macro_rules! check_format {
    ($format:expr) => {
        if !wrapper::is_format_avail($format) {
            return Err(WindowsError(0))
        }
    }
}

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
        const FMT_FLAGS: DWORD = FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ARGUMENT_ARRAY;
        let mut format_buff: [u16; BUF_SIZE] = [0; BUF_SIZE];
        let num_chars: u32 = unsafe { FormatMessageW(FMT_FLAGS,
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

macro_rules! impl_traits
{
    ($($t:ty), +) => {
        $(
            impl From<$t> for WindowsError {
                fn from(num: $t) -> Self {
                    WindowsError(num as u32)
                }
            }

            impl Into<$t> for WindowsError {
                fn into(self) -> $t {
                    self.0 as $t
                }
            }

            impl PartialEq<$t> for WindowsError {
                fn eq(&self, right: &$t) -> bool {
                    self.0 == *right as u32
                }

                fn ne(&self, right: &$t) -> bool {
                    self.0 != *right as u32
                }
            }
        )+
    };
}

impl_traits!(u32, u16, u8, usize, i32, i16, i8, isize);

#[inline]
///Set clipboard with text.
pub fn set_clipboard<T: ?Sized + AsRef<std::ffi::OsStr>>(text: &T) -> Result<(), WindowsError> {
    try!(wrapper::open_clipboard());
    let result = wrapper::set_clipboard(text);
    try!(wrapper::close_clipboard());
    result
}

#[inline]
///Retrieves clipboard content in UTF16 format and convert it to String.
///
///# Return result:
///
///* ```Ok``` Content of clipboard which is stored in ```String```.
///* ```Err``` Contains ```WindowsError```.
pub fn get_clipboard_string() -> Result<String, WindowsError> {
    //If there is no such format on clipboard then we can return right now
    //From point of view Windows this is not an error case, but we still unable to get anything.
    check_format!(clipboard_formats::CF_UNICODETEXT);

    try!(wrapper::open_clipboard());
    let result = wrapper::get_clipboard_string();
    try!(wrapper::close_clipboard());
    result
}

#[inline]
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
    //see comment get_clipboard_string()
    check_format!(format);

    try!(wrapper::open_clipboard());
    let result = wrapper::get_clipboard(format);
    try!(wrapper::close_clipboard());
    result
}

#[inline]
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

#[inline]
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
    try_none!(wrapper::open_clipboard());
    let result = wrapper::get_format_name(format);
    try_none!(wrapper::close_clipboard());
    result
}
