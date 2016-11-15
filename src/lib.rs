#![cfg(windows)]
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
extern crate windows_error;

use windows_error::WindowsError;

mod constants;
pub mod clipboard_formats;

pub mod wrapper;

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
            return Err(WindowsError::new(0))
        }
    }
}

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

//Re-export functions that do not require open/close
pub use wrapper::{get_format_name,
                  count_formats,
                  register_format,
                  is_format_avail};
