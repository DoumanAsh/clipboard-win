//!Raw bindings to Windows clipboard.
//!
//!## General information
//!
//!All pre & post conditions are stated in description of functions.
//!
//!### Open clipboard
//! To access any information inside clipboard it is necessary to open it by means of
//! [open()](fn.open.html).
//!
//! After that Clipboard cannot be opened any more until [close()](fn.close.html) is called.

use winapi::um::winuser::{OpenClipboard, CloseClipboard, EmptyClipboard, GetClipboardSequenceNumber, GetClipboardData, IsClipboardFormatAvailable, CountClipboardFormats, EnumClipboardFormats, GetClipboardFormatNameW, RegisterClipboardFormatW};
use winapi::um::winbase::{GlobalSize, GlobalLock, GlobalUnlock};
use winapi::um::errhandlingapi::GetLastError;
use winapi::ctypes::{c_int, c_uint};
use winapi::um::stringapiset::{MultiByteToWideChar, WideCharToMultiByte};
use winapi::um::winnls::CP_UTF8;

use str_buf::StrBuf;

use core::{slice, mem, ptr};
use core::ffi::c_void;
use core::num::NonZeroU32;

use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::format;
use crate::formats;

#[inline(always)]
///Returns last winapi error code.
pub fn last_error() -> winapi::ctypes::c_ulong {
    unsafe {
        GetLastError()
    }
}

#[inline]
///Opens clipboard.
///
///Wrapper around ```OpenClipboard```.
///
///# Pre-conditions:
///
///* Clipboard is not opened yet.
///
///# Post-conditions (if successful):
///
///* Clipboard can be accessed for read and write operations.
///
///# Returns
///
///- `true` Successfully opened clipboard.
///` `false` Failed to open clipboard, check last IO error for reason.
pub fn open() -> bool {
    unsafe {
        OpenClipboard(ptr::null_mut()) != 0
    }
}

#[inline]
///Closes clipboard.
///
///Wrapper around ```CloseClipboard```.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
///
///# Returns
///
///- `true` Successfully closed clipboard.
///` `false` Failed to open clipboard, check last IO error for reason.
pub fn close() -> bool {
    unsafe {
        CloseClipboard() != 0
    }
}

#[inline]
///Empties clipboard.
///
///Wrapper around ```EmptyClipboard```.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
///
///# Returns
///
///- `true` Successfully emptied clipboard.
///` `false` Failed to open clipboard, check last IO error for reason.
pub fn empty() -> bool {
    unsafe {
        EmptyClipboard() != 0
    }
}

#[inline]
///Retrieves clipboard sequence number.
///
///Wrapper around ```GetClipboardSequenceNumber```.
///
///# Returns:
///
///* ```Some``` Contains return value of ```GetClipboardSequenceNumber```.
///* ```None``` In case if you do not have access. It means that zero is returned by system.
pub fn seq_num() -> Option<u32> {
    match unsafe { GetClipboardSequenceNumber() } {
        0 => None,
        num => Some(num)
    }
}

#[inline]
///Retrieves size of clipboard data for specified format.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
///
///# Returns:
///
///Size in bytes if format presents on clipboard.
///
///# Unsafety:
///
///In some cases, clipboard content might be so invalid that it crashes on `GlobalSize` (e.g.
///Bitmap)
///
///Due to that function is marked as unsafe
pub unsafe fn size_unsafe(format: u32) -> Option<usize> {
    let clipboard_data = GetClipboardData(format);

    match clipboard_data.is_null() {
        false => Some(GlobalSize(clipboard_data) as usize),
        true => None,
    }
}

#[inline]
///Retrieves size of clipboard data for specified format.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
///
///# Returns:
///
///Size in bytes if format presents on clipboard.
pub fn size(format: u32) -> Option<usize> {
    let clipboard_data = unsafe {GetClipboardData(format)};

    if clipboard_data.is_null() {
        return None
    }

    unsafe {
        if GlobalLock(clipboard_data).is_null() {
            return None;
        }

        let result = Some(GlobalSize(clipboard_data) as usize);

        GlobalUnlock(clipboard_data);

        result
    }
}

#[inline(always)]
///Retrieves raw pointer to clipboard data.
///
///Wrapper around ```GetClipboardData```.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
pub fn get_clipboard_data(format: c_uint) -> *mut c_void {
    unsafe { GetClipboardData(format) as *mut c_void }
}

#[inline(always)]
///Determines whenever provided clipboard format is available on clipboard or not.
pub fn is_format_avail(format: c_uint) -> bool {
    unsafe { IsClipboardFormatAvailable(format) != 0 }
}

#[inline]
///Retrieves number of currently available formats on clipboard.
///
///Returns `None` if `CountClipboardFormats` failed.
pub fn count_formats() -> Option<usize> {
    let result = unsafe { CountClipboardFormats() };

    if result == 0 {
        if last_error() != 0 {
            return None
        }
    }

    Some(result as usize)
}

///Enumerator over available clipboard formats.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
pub struct EnumFormats {
    idx: u32
}

impl EnumFormats {
    /// Constructs enumerator over all available formats.
    pub fn new() -> EnumFormats {
        EnumFormats { idx: 0 }
    }

    /// Constructs enumerator that starts from format.
    pub fn from(format: u32) -> EnumFormats {
        EnumFormats { idx: format }
    }

    /// Resets enumerator to list all available formats.
    pub fn reset(&mut self) -> &EnumFormats {
        self.idx = 0;
        self
    }
}

impl Iterator for EnumFormats {
    type Item = u32;

    /// Returns next format on clipboard.
    ///
    /// In case of failure (e.g. clipboard is closed) returns `None`.
    fn next(&mut self) -> Option<u32> {
        self.idx = unsafe { EnumClipboardFormats(self.idx) };

        if self.idx == 0 {
            None
        } else {
            Some(self.idx)
        }
    }

    /// Relies on `count_formats` so it is only reliable
    /// when hinting size for enumeration of all formats.
    ///
    /// Doesn't require opened clipboard.
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, count_formats())
    }
}

macro_rules! match_format_name_big {
    ( $name:expr, $( $f:ident ),* ) => {
        match $name {
            $( formats::$f => Some(stringify!($f).to_owned()),)*
            formats::CF_GDIOBJFIRST ..= formats::CF_GDIOBJLAST => Some(format!("CF_GDIOBJ{}", $name - formats::CF_GDIOBJFIRST)),
            formats::CF_PRIVATEFIRST ..= formats::CF_PRIVATELAST => Some(format!("CF_PRIVATE{}", $name - formats::CF_PRIVATEFIRST)),
            _ => {
                let format_buff = [0u16; 256];
                unsafe {
                    let buff_p = format_buff.as_ptr() as *mut u16;

                    match GetClipboardFormatNameW($name, buff_p, format_buff.len() as c_int) {
                        0 => None,
                        size => Some(String::from_utf16_lossy(&format_buff[..size as usize])),
                    }
                }
            }
        }
    }
}

macro_rules! match_format_name {
    ( $name:expr, $( $f:ident ),* ) => {
        use core::fmt::Write;
        let mut result = StrBuf::<[u8; 52]>::new();

        match $name {
            $( formats::$f => {
                let _ = result.push_str(stringify!($f));
            },)*
            formats::CF_GDIOBJFIRST ..= formats::CF_GDIOBJLAST => {
                let _ = write!(result, "CF_GDIOBJ{}", $name - formats::CF_GDIOBJFIRST);
            },
            formats::CF_PRIVATEFIRST ..= formats::CF_PRIVATELAST => {
                let _ = write!(result, "CF_PRIVATE{}", $name - formats::CF_PRIVATEFIRST);
            },
            _ => {
                let mut format_buff = [0u16; 52];
                unsafe {
                    let buff_p = format_buff.as_mut_ptr() as *mut u16;
                    match GetClipboardFormatNameW($name, buff_p, format_buff.len() as c_int) {
                        0 => return None,
                        len => match WideCharToMultiByte(winapi::um::winnls::CP_UTF8, 0, format_buff.as_ptr(), len, result.as_ptr() as *mut i8, result.remaining() as i32, ptr::null(), ptr::null_mut()) {
                            0 => return None,
                            len => result.set_len(len as u8),
                        }
                    }
                }
            }
        }

        return Some(result)
    }
}

///Returns format name based on it's code.
///
///# Parameters:
///
///* ```format``` clipboard format code.
///
///# Return result:
///
///* ```Some``` Name of valid format.
///* ```None``` Format is invalid or doesn't exist.
pub fn format_name(format: u32) -> Option<StrBuf<[u8; 52]>> {
    match_format_name!(format,
                       CF_BITMAP,
                       CF_DIB,
                       CF_DIBV5,
                       CF_DIF,
                       CF_DSPBITMAP,
                       CF_DSPENHMETAFILE,
                       CF_DSPMETAFILEPICT,
                       CF_DSPTEXT,
                       CF_ENHMETAFILE,
                       CF_HDROP,
                       CF_LOCALE,
                       CF_METAFILEPICT,
                       CF_OEMTEXT,
                       CF_OWNERDISPLAY,
                       CF_PALETTE,
                       CF_PENDATA,
                       CF_RIFF,
                       CF_SYLK,
                       CF_TEXT,
                       CF_WAVE,
                       CF_TIFF,
                       CF_UNICODETEXT);
}

///Returns format name based on it's code (allocating variant suitable for big names)
///
///# Parameters:
///
///* ```format``` clipboard format code.
///
///# Return result:
///
///* ```Some``` Name of valid format.
///* ```None``` Format is invalid or doesn't exist.
pub fn format_name_big(format: u32) -> Option<String> {
    match_format_name_big!(format,
                           CF_BITMAP,
                           CF_DIB,
                           CF_DIBV5,
                           CF_DIF,
                           CF_DSPBITMAP,
                           CF_DSPENHMETAFILE,
                           CF_DSPMETAFILEPICT,
                           CF_DSPTEXT,
                           CF_ENHMETAFILE,
                           CF_HDROP,
                           CF_LOCALE,
                           CF_METAFILEPICT,
                           CF_OEMTEXT,
                           CF_OWNERDISPLAY,
                           CF_PALETTE,
                           CF_PENDATA,
                           CF_RIFF,
                           CF_SYLK,
                           CF_TEXT,
                           CF_WAVE,
                           CF_TIFF,
                           CF_UNICODETEXT)
}

#[inline]
///Registers a new clipboard format with specified name as C wide string (meaning it must have null
///char at the end).
///
///# Returns:
///
///Newly registered format identifier, if successful.
///
///# Note:
///
///Custom format identifier is in range `0xC000...0xFFFF`.
pub unsafe fn register_raw_format(name: &[u16]) -> Option<NonZeroU32> {
    debug_assert_eq!(name[name.len()-1], b'\0' as u16);
    NonZeroU32::new(RegisterClipboardFormatW(name.as_ptr()) )
}

///Registers a new clipboard format with specified name.
///
///# Returns:
///
///Newly registered format identifier, if successful.
///
///# Note:
///
///Custom format identifier is in range `0xC000...0xFFFF`.
pub fn register_format(name: &str) -> Option<NonZeroU32> {
    let size = unsafe {
        MultiByteToWideChar(CP_UTF8, 0, name.as_ptr() as *const _, name.len() as c_int, ptr::null_mut(), 0)
    };

    if size == 0 {
        return None;
    }

    if size > 52 {
        let mut buffer = alloc::vec::Vec::with_capacity(size as usize);
        let size = unsafe {
            MultiByteToWideChar(CP_UTF8, 0, name.as_ptr() as *const _, name.len() as c_int, buffer.as_mut_ptr(), size)
        };
        unsafe {
            buffer.set_len(size as usize);
            buffer.push(0);
            register_raw_format(&buffer)
        }
    } else {
        let mut buffer = mem::MaybeUninit::<[u16; 52]>::uninit();
        let size = unsafe {
            MultiByteToWideChar(CP_UTF8, 0, name.as_ptr() as *const _, name.len() as c_int, buffer.as_mut_ptr() as *mut u16, 51)
        };
        unsafe {
            ptr::write((buffer.as_mut_ptr() as *mut u16).offset(size as isize), 0);
            register_raw_format(slice::from_raw_parts(buffer.as_ptr() as *const u16, size as usize + 1))
        }
    }
}
