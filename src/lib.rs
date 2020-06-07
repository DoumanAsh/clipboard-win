#![cfg(windows)]
//! This crate provide simple means to operate with Windows clipboard.
//!
//!# Note keeping Clipboard around:
//!
//! In Windows [Clipboard](struct.Clipboard.html) opens globally and only one application can set data onto format at the time.
//!
//! Therefore as soon as operations are finished, user is advised to close [Clipboard](struct.Clipboard.html).
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
//! use clipboard_win::Clipboard;
//!
//! fn main() {
//!     Clipboard::new().unwrap().empty();
//! }
//! ```
//!## Set and get raw data
//! ```rust
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
//! ```
//!
//!## Set and get String
//!
//!```rust
//!use clipboard_win::Clipboard;
//!
//!use std::str;
//!
//!fn main() {
//!    let text = "For my waifu!";
//!    Clipboard::new().unwrap().set_string(text);
//!
//!    let mut result = String::new();
//!    Clipboard::new().unwrap().get_string(&mut result).unwrap();
//!    assert_eq!(text, result);
//!}
//!```
//!
//!## Set and get String shortcuts
//!
//!```rust
//!use clipboard_win::{set_clipboard_string, get_clipboard_string};
//!
//!use std::str;
//!
//!fn main() {
//!    let text = "For my waifu!";
//!    set_clipboard_string(text).expect("Success");
//!
//!    let result = get_clipboard_string().unwrap();
//!    assert_eq!(text, result);
//!}
//!```

#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

pub mod formats;
pub mod raw;
pub mod dib;
pub mod image;
pub mod utils;

use std::{io};
use std::path::PathBuf;

///Clipboard accessor.
///
///# Note:
///
///You can have only one such accessor across your program.
///
///# Warning:
///
///In Windows Clipboard opens globally and only one application can set data
///onto format at the time.
///
///Therefore as soon as operations are finished, user is advised to close Clipboard.
pub struct Clipboard {
    inner: ()
}

impl Clipboard {
    ///Initializes new clipboard accessor.
    ///
    ///Attempts to open clipboard.
    #[inline]
    pub fn new() -> io::Result<Clipboard> {
        raw::open().map(|_| Clipboard { inner: () })
    }

    #[inline]
    ///Attempts to initialize clipboard `num` times before giving up.
    pub fn new_attempts(mut num: usize) -> io::Result<Clipboard> {
        loop {
            match raw::open() {
                Ok(_) => break Ok(Clipboard { inner: () }),
                Err(error) => match num {
                    0 => break Err(error),
                    _ => num -= 1
                },
            }
        }
    }

    ///Empties clipboard.
    #[inline]
    pub fn empty(&self) -> io::Result<&Clipboard> {
        raw::empty().map(|_| self)
    }

    ///Retrieves size of clipboard content.
    #[inline]
    pub fn size(&self, format: u32) -> Option<usize> {
        raw::size(format)
    }

    ///Sets data onto clipboard with specified format.
    ///
    ///Wraps `raw::set()`
    #[inline]
    pub fn set(&self, format: u32, data: &[u8]) -> io::Result<()> {
        raw::set(format, data)
    }

    ///Sets `str` or `String` onto clipboard as Unicode format.
    ///
    ///Under hood it transforms Rust `UTF-8` String into `UTF-16`
    #[inline]
    pub fn set_string(&self, text: &str) -> io::Result<()> {
        raw::set_string(text)
    }

    ///Retrieves data of specified format from clipboard.
    ///
    ///Wraps `raw::get()`
    #[inline]
    pub fn get(&self, format: u32, data: &mut [u8]) -> io::Result<usize> {
        raw::get(format, data)
    }

    ///Retrieves `String` of `CF_UNICODETEXT` format from clipboard.
    ///
    ///Wraps `raw::get_string()`
    #[inline]
    pub fn get_string(&self, storage: &mut String) -> io::Result<()> {
        raw::get_string(storage)
    }

    /// Retrieves a list of file paths from the `CF_HDROP` format from the clipboard.
    ///
    /// Wraps `raw::get_file_list()`
    #[inline]
    pub fn get_file_list(&self) -> io::Result<Vec<PathBuf>> {
        raw::get_file_list()
    }

    #[inline]
    ///Retrieves `Bitmap` of `CF_DIB`
    pub fn get_dib(&self) -> io::Result<dib::Image> {
        raw::get_clipboard_data(formats::CF_DIB).or_else(|_| raw::get_clipboard_data(formats::CF_DIBV5))
                                                .and_then(|handle| dib::Image::from_handle(handle.as_ptr()))
    }

    #[inline]
    ///Retrieves `Bitmap` of `CF_BITMAP`
    pub fn get_bitmap(&self) -> io::Result<image::Image> {
        raw::get_clipboard_data(formats::CF_BITMAP).and_then(|handle| image::Image::from_handle(handle))
    }

    #[inline]
    ///Sets bitmap image onto clipboard as `CF_BITMAP`
    pub fn set_bitmap(&self, image: &image::Image) -> io::Result<()> {
        image.write_to_clipboard()
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

///Shortcut to retrieve string from clipboard.
///
///It opens clipboard and gets string, if possible.
#[inline]
pub fn get_clipboard_string() -> io::Result<String> {
    let mut data = String::new();
    Clipboard::new_attempts(10)?.get_string(&mut data).map(|_| data)
}

///Shortcut to set string onto clipboard.
///
///It opens clipboard and attempts to set string.
#[inline]
pub fn set_clipboard_string(data: &str) -> io::Result<()> {
    Clipboard::new_attempts(10)?.set_string(data)
}
