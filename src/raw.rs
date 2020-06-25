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

use winapi::um::winuser::{OpenClipboard, CloseClipboard, EmptyClipboard, GetClipboardSequenceNumber, GetClipboardData, IsClipboardFormatAvailable, CountClipboardFormats, EnumClipboardFormats, GetClipboardFormatNameW, RegisterClipboardFormatW, SetClipboardData};
use winapi::um::winbase::{GlobalSize, GlobalLock, GlobalUnlock};
use winapi::ctypes::{c_int, c_uint, c_void};
use winapi::um::stringapiset::{MultiByteToWideChar, WideCharToMultiByte};
use winapi::um::winnls::CP_UTF8;

use str_buf::StrBuf;
use error_code::SystemError;

use core::{slice, mem, ptr, cmp};
use core::num::{NonZeroUsize, NonZeroU32};

use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::format;

use crate::{SysResult, formats};
use crate::utils::{WinMem};

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
pub fn open() -> SysResult<()> {
    match unsafe { OpenClipboard(ptr::null_mut()) } {
        0 => Err(SystemError::last()),
        _ => Ok(()),
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
pub fn close() -> SysResult<()> {
    match unsafe { CloseClipboard() } {
        0 => Err(SystemError::last()),
        _ => Ok(()),
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
pub fn empty() -> SysResult<()> {
    match unsafe { EmptyClipboard() } {
        0 => Err(SystemError::last()),
        _ => Ok(()),
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
pub fn seq_num() -> Option<NonZeroU32> {
    unsafe { NonZeroU32::new(GetClipboardSequenceNumber()) }
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
pub unsafe fn size_unsafe(format: u32) -> Option<NonZeroUsize> {
    let clipboard_data = GetClipboardData(format);

    match clipboard_data.is_null() {
        false => NonZeroUsize::new(GlobalSize(clipboard_data) as usize),
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
///Size in bytes if format is presents on clipboard.
pub fn size(format: u32) -> Option<NonZeroUsize> {
    let clipboard_data = unsafe {GetClipboardData(format)};

    if clipboard_data.is_null() {
        return None
    }

    unsafe {
        if GlobalLock(clipboard_data).is_null() {
            return None;
        }

        let result = NonZeroUsize::new(GlobalSize(clipboard_data) as usize);

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
pub fn get_clipboard_data(format: c_uint) -> SysResult<ptr::NonNull<c_void>> {
    let ptr = unsafe { GetClipboardData(format) as *mut c_void };
    match ptr::NonNull::new(ptr) {
        Some(ptr) => Ok(ptr),
        None => Err(SystemError::last()),
    }
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
        if !SystemError::last().is_zero() {
            return None
        }
    }

    Some(result as usize)
}

///Copies raw bytes from clipboard with specified `format`
///
///Returns number of copied bytes on success, otherwise 0.
///
///It is safe to pass uninit memory
pub fn get(format: u32, out: &mut [u8]) -> SysResult<usize> {
    let size = out.len();
    debug_assert!(size > 0);
    let out_ptr = out.as_mut_ptr();

    let ptr = WinMem::from_borrowed(get_clipboard_data(format)?);

    let result = unsafe {
        let (data_ptr, _lock) = ptr.lock()?;
        let data_size = cmp::min(GlobalSize(ptr.get()) as usize, size);
        ptr::copy_nonoverlapping(data_ptr.as_ptr() as *const u8, out_ptr, data_size);
        data_size
    };

    Ok(result)
}

///Copies raw bytes from clipboard with specified `format`, appending to `out` buffer.
///
///Returns number of copied bytes on success, otherwise 0.
pub fn get_vec(format: u32, out: &mut alloc::vec::Vec<u8>) -> SysResult<usize> {
    let ptr = WinMem::from_borrowed(get_clipboard_data(format)?);

    let result = unsafe {
        let (data_ptr, _lock) = ptr.lock()?;
        let data_size = GlobalSize(ptr.get()) as usize;

        out.reserve(data_size as usize);
        let storage_cursor = out.len();
        let storage_ptr = out.as_mut_ptr().add(out.len()) as *mut _;

        ptr::copy_nonoverlapping(data_ptr.as_ptr() as *const u8, storage_ptr, data_size);
        out.set_len(storage_cursor + data_size as usize);

        data_size
    };

    Ok(result)
}

///Copies raw bytes onto clipboard with specified `format`, returning whether it was successful.
pub fn set(format: u32, data: &[u8]) -> SysResult<()> {
    let size = data.len();
    debug_assert!(size > 0);

    let mem = WinMem::new_global_mem(size)?;

    {
        let (ptr, _lock) = mem.lock()?;
        unsafe { ptr::copy_nonoverlapping(data.as_ptr(), ptr.as_ptr() as _, size) };
    }

    let _ = empty();

    if unsafe { !SetClipboardData(format, mem.get()).is_null() } {
        //SetClipboardData takes ownership
        mem.release();
        return Ok(());
    }

    Err(error_code::SystemError::last())
}

///Copies raw bytes from clipboard with specified `format`, appending to `out` buffer.
///
///Returns number of copied bytes on success, otherwise 0.
pub fn get_string(out: &mut alloc::vec::Vec<u8>) -> SysResult<usize> {
    let ptr = WinMem::from_borrowed(get_clipboard_data(formats::CF_UNICODETEXT)?);

    let result = unsafe {
        let (data_ptr, _lock) = ptr.lock()?;
        let data_size = GlobalSize(ptr.get()) as usize / mem::size_of::<u16>();
        let storage_req_size = WideCharToMultiByte(CP_UTF8, 0, data_ptr.as_ptr() as _, data_size as _, ptr::null_mut(), 0, ptr::null(), ptr::null_mut());

        if storage_req_size == 0 {
            return Err(SystemError::last());
        }

        let storage_cursor = out.len();
        out.reserve(storage_req_size as usize);
        let storage_ptr = out.as_mut_ptr().add(storage_cursor) as *mut _;
        WideCharToMultiByte(CP_UTF8, 0, data_ptr.as_ptr() as _, data_size as _, storage_ptr, storage_req_size, ptr::null(), ptr::null_mut());
        out.set_len(storage_cursor + storage_req_size as usize);

        //It seems WinAPI always supposed to have at the end null char.
        //But just to be safe let's check for it and only then remove.
        if let Some(null_idx) = out.iter().skip(storage_cursor).position(|b| *b == b'\0') {
            out.set_len(storage_cursor + null_idx);
        }

        out.len() - storage_cursor
    };

    Ok(result)
}

///Copies unicode string onto clipboard, performing necessary conversions, returning true on
///success.
pub fn set_string(data: &str) -> SysResult<()> {
    debug_assert!(data.len() > 0);

    let size = unsafe {
        MultiByteToWideChar(CP_UTF8, 0, data.as_ptr() as *const _, data.len() as _, ptr::null_mut(), 0)
    };

    if size != 0 {
        let mem = WinMem::new_global_mem((mem::size_of::<u16>() * (size as usize + 1)) as _)?;
        {
            let (ptr, _lock) = mem.lock()?;
            let ptr = ptr.as_ptr() as *mut u16;
            unsafe {
                MultiByteToWideChar(CP_UTF8, 0, data.as_ptr() as *const _, data.len() as _, ptr, size);
                ptr::write(ptr.offset(size as isize), 0);
            }
        }

        let _ = empty();

        if unsafe { !SetClipboardData(formats::CF_UNICODETEXT, mem.get()).is_null() } {
            //SetClipboardData takes ownership
            mem.release();
            return Ok(());
        }
    }

    Err(error_code::SystemError::last())
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
