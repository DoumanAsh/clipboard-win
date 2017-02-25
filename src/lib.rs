#![cfg(windows)]
//! This crate provide simple means to operate with Windows clipboard.
//!
//!# Clipboard
//!
//! All read and write access to Windows clipboard requires user to open it.
//!
//! For your convenience you can use [Clipboard](struct.Clipboard.html) struct that opens it at creation
//! and closes on its  drop.
//!
//! Alternatively you can access all functionality directly through [raw module](raw/index.html).
//!
//! Below you can find examples of usage.
//!
//!## Empty clipboard
//!
//! ```rust
//! extern crate clipboard_win;
//!
//! use clipboard_win::Clipboard;
//!
//! fn main() {
//!     Clipboard::new().unwrap().empty();
//! }
//! ```
//!## Set and get raw data
//! ```rust
//! extern crate clipboard_win;
//! use clipboard_win::formats;
//!
//! use clipboard_win::Clipboard;
//!
//! use std::str;
//!
//! fn main() {
//!     let text = "For my waifu!\0"; //For text we need to pass C-like string
//!     Clipboard::new().unwrap().set(formats::CF_TEXT, text.as_bytes());
//!
//!     let mut buffer = [0u8; 52];
//!     let result = Clipboard::new().unwrap().get(formats::CF_TEXT, &mut buffer).unwrap();
//!     assert_eq!(str::from_utf8(&buffer[..result]).unwrap(), text);
//! }

extern crate winapi;
extern crate user32;
extern crate kernel32;

use std::io;

mod utils;
pub mod formats;
pub mod raw;

pub use raw::{
    register_format
};

///Clipboard accessor.
///
///# Note:
///
///You can have only one such accessor across your program.
pub struct Clipboard {
    inner: ()
}

impl Clipboard {
    ///Initializes new clipboard accessor.
    ///
    ///Attempts to open clipboard.
    #[inline]
    pub fn new() -> io::Result<Clipboard> {
        raw::open().map(|_| Clipboard {inner: ()})
    }

    ///Empties clipboard.
    #[inline]
    pub fn empty(&self) -> io::Result<&Clipboard> {
        raw::empty().map(|_| self)
    }

    ///Retrieves size of clipboard content.
    #[inline]
    pub fn size(&self, format: u32) -> Option<u64> {
        raw::size(format)
    }

    ///Sets data onto clipboard with specified format.
    ///
    ///Wraps `raw::set()`
    #[inline]
    pub fn set(&self, format: u32, data: &[u8]) -> io::Result<&Clipboard> {
        raw::set(format, data).map(|_| self)
    }

    ///Retrieves data of specified format from clipboard.
    ///
    ///Wraps `raw::get()`
    #[inline]
    pub fn get(&self, format: u32, data: &mut [u8]) -> io::Result<usize> {
        raw::get(format, data)
    }

    ///Enumerator over all formats on clipboard..
    #[inline]
    pub fn enum_formats(&self) -> raw::EnumFormats {
        raw::EnumFormats::new()
    }

    ///Returns Clipboard sequence number.
    #[inline]
    pub fn seq_num() -> Option<u32> {
        raw::seq_num()
    }

    ///Determines whenever provided clipboard format is available on clipboard or not.
    #[inline]
    pub fn is_format_avail(format: u32) -> bool {
        raw::is_format_avail(format)
    }

    ///Retrieves number of currently available formats on clipboard.
    #[inline]
    pub fn count_formats() -> io::Result<i32> {
        raw::count_formats()
    }

}

impl Drop for Clipboard {
    fn drop(&mut self) {
        let _ = raw::close();
        self.inner
    }
}
