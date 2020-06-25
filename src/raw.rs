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

use core::{mem, ptr, cmp};
use std::ffi::OsString;
use std::os::windows::ffi::{
    OsStrExt,
    OsStringExt
};
use std::os::raw::{
    c_int,
    c_uint,
    c_void,
};
use std::path::PathBuf;
use std::io;

use crate::formats;
use crate::utils::LockedData;

use winapi::shared::basetsd::{
    SIZE_T
};

use winapi::um::shellapi::{
    DragQueryFileW,
    HDROP
};

use winapi::um::winbase::{
    GlobalSize,
    GlobalLock,
    GlobalUnlock,
    GlobalAlloc,
    GlobalFree
};

use winapi::um::winuser::{
    OpenClipboard,
    CloseClipboard,
    EmptyClipboard,
    GetClipboardSequenceNumber,
    CountClipboardFormats,
    IsClipboardFormatAvailable,
    EnumClipboardFormats,
    RegisterClipboardFormatW,
    GetClipboardFormatNameW,
    GetClipboardData,
    SetClipboardData
};

const GHND: c_uint = 0x42;

#[inline]
///Opens clipboard.
///
///Wrapper around ```OpenClipboard```.
///
///# Pre-conditions:
///
///* Clipboard is not opened yet.
///
///# Post-conditions:
///
///* Clipboard can be accessed for read and write operations.
pub fn open() -> io::Result<()> {
    unsafe {
        if OpenClipboard(ptr::null_mut()) == 0 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(())
}

#[inline]
///Closes clipboard.
///
///Wrapper around ```CloseClipboard```.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
pub fn close() -> io::Result<()> {
    unsafe {
        if CloseClipboard() == 0 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(())
}

#[inline]
///Empties clipboard.
///
///Wrapper around ```EmptyClipboard```.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
pub fn empty() -> io::Result<()> {
    unsafe {
        if EmptyClipboard() == 0 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(())
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
    let result: u32 = unsafe { GetClipboardSequenceNumber() };

    if result == 0 {
        return None;
    }

    Some(result)
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

#[inline]
///Retrieves raw pointer to clipboard data.
///
///Wrapper around ```GetClipboardData```.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
pub fn get_clipboard_data(format: c_uint) -> io::Result<ptr::NonNull<c_void>> {
    let clipboard_data = unsafe { GetClipboardData(format) };

    match ptr::NonNull::new(clipboard_data as *mut c_void) {
        Some(ptr) => Ok(ptr),
        None => Err(io::Error::last_os_error()),
    }
}

///Retrieves data of specified format from clipboard.
///
///Wrapper around ```GetClipboardData```.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
///
///# Note:
///
///Clipboard data is truncated by the size of provided storage.
///
///# Returns:
///
///Number of copied bytes.
pub fn get(format: u32, result: &mut [u8]) -> io::Result<usize> {
    let clipboard_data = get_clipboard_data(format as c_uint)?;

    unsafe {
        let (data_ptr, _guard) = LockedData::new(clipboard_data.as_ptr())?;

        let data_size = cmp::min(GlobalSize(clipboard_data.as_ptr()) as usize, result.len());

        ptr::copy_nonoverlapping(data_ptr.as_ptr(), result.as_mut_ptr(), data_size);

        Ok(data_size)
    }
}

///Retrieves String from `CF_UNICODETEXT` format
///
///Specialized version of [get](fn.get.html) to avoid unnecessary allocations.
///
///# Note:
///
///Usually WinAPI returns strings with null terminated character at the end.
///This character is trimmed.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
pub fn get_string(storage: &mut String) -> io::Result<()> {
    use winapi::um::stringapiset::WideCharToMultiByte;
    use winapi::um::winnls::CP_UTF8;

    let clipboard_data = get_clipboard_data(formats::CF_UNICODETEXT)?;

    unsafe {
        let (data_ptr, _guard) = LockedData::new(clipboard_data.as_ptr())?;

        let data_size = GlobalSize(clipboard_data.as_ptr()) as usize / std::mem::size_of::<u16>();
        let storage_req_size = WideCharToMultiByte(CP_UTF8, 0, data_ptr.as_ptr(), data_size as c_int, ptr::null_mut(), 0, ptr::null(), ptr::null_mut());
        if storage_req_size == 0 {
            return Err(io::Error::last_os_error());
        }

        {
            storage.reserve(storage_req_size as usize);
            let storage = storage.as_mut_vec();
            let storage_cursor = storage.len();
            let storage_ptr = storage.as_mut_ptr().add(storage_cursor) as *mut _;
            WideCharToMultiByte(CP_UTF8, 0, data_ptr.as_ptr(), data_size as c_int, storage_ptr, storage_req_size, ptr::null(), ptr::null_mut());
            storage.set_len(storage_cursor + storage_req_size as usize);
        }

        //It seems WinAPI always supposed to have at the end null char.
        //But just to be safe let's check for it and only then remove.
        if let Some(null_idx) = storage.find('\0') {
            storage.drain(null_idx..);
        }

        Ok(())
    }
}

/// Retrieves a list of file paths from the `CF_HDROP` format.
///
/// # Pre-conditions:
///
/// * [open()](fn.open.html) has been called.
pub fn get_file_list() -> io::Result<Vec<PathBuf>> {
    let clipboard_data = get_clipboard_data(formats::CF_HDROP)?;
    let clipboard_data = clipboard_data.as_ptr();

    unsafe {
        let (_, _locked_data) = LockedData::new::<c_void>(clipboard_data)?;

        let num_files = DragQueryFileW(clipboard_data as HDROP, std::u32::MAX, ptr::null_mut(), 0);

        let mut file_names = Vec::with_capacity(num_files as usize);

        for file_index in 0..num_files {
            let required_size_no_null = DragQueryFileW(clipboard_data as HDROP, file_index, ptr::null_mut(), 0);
            if required_size_no_null == 0 {
                return Err(io::ErrorKind::Other.into());
            }

            let required_size = required_size_no_null + 1;
            let mut file_str_buf = Vec::with_capacity(required_size as usize);

            if DragQueryFileW(clipboard_data as HDROP, file_index, file_str_buf.as_mut_ptr(), required_size) == 0 {
                return Err(io::ErrorKind::Other.into());
            }

            file_str_buf.set_len(required_size as usize);
            // Remove terminating zero
            let os_string = OsString::from_wide(&file_str_buf[..required_size_no_null as usize]);
            file_names.push(PathBuf::from(os_string));
        }

        Ok(file_names)
    }
}

///Sets data onto clipboard with specified format.
///
///Wrapper around ```SetClipboardData```.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
///
///# Panic
///
///On zero input. If you want to empty clipboard then use [empty()](fn.empty.html).
pub fn set(format: u32, data: &[u8]) -> io::Result<()> {
    debug_assert!(data.len() > 0);

    let size = data.len();

    let alloc_handle = unsafe { GlobalAlloc(GHND, size as SIZE_T) };

    if alloc_handle.is_null() {
        Err(io::Error::last_os_error())
    }
    else {
        unsafe {
            {
                let (ptr, _lock) = LockedData::new(alloc_handle)?;
                ptr::copy_nonoverlapping(data.as_ptr(), ptr.as_ptr(), size);
            }
            EmptyClipboard();

            if SetClipboardData(format, alloc_handle).is_null() {
                let result = io::Error::last_os_error();
                GlobalFree(alloc_handle);
                Err(result)
            }
            else {
                Ok(())
            }
        }
    }
}

///Sets data onto clipboard with specified format.
///
///Wrapper around ```SetClipboardData```.
///
///# Pre-conditions:
///
///* [open()](fn.open.html) has been called.
///
///# Panic
///
///On zero input. If you want to empty clipboard then use [empty()](fn.empty.html).
pub fn set_string(data: &str) -> io::Result<()> {
    use winapi::um::stringapiset::MultiByteToWideChar;
    use winapi::um::winnls::CP_UTF8;

    debug_assert!(data.len() > 0);

    let size = unsafe {
        MultiByteToWideChar(CP_UTF8, 0, data.as_ptr() as *const _, data.len() as c_int, ptr::null_mut(), 0)
    };

    if size == 0 {
        return Err(io::Error::last_os_error())
    }

    let alloc_handle = unsafe { GlobalAlloc(GHND, mem::size_of::<u16>() * (size as SIZE_T + 1)) };

    if alloc_handle.is_null() {
        Err(io::Error::last_os_error())
    }
    else {
        unsafe {
            {
                let (ptr, _lock) = LockedData::new(alloc_handle)?;
                MultiByteToWideChar(CP_UTF8, 0, data.as_ptr() as *const _, data.len() as c_int, ptr.as_ptr(), size);
                ptr::write(ptr.as_ptr().offset(size as isize), 0);
            }
            EmptyClipboard();

            if SetClipboardData(formats::CF_UNICODETEXT, alloc_handle).is_null() {
                let result = io::Error::last_os_error();
                GlobalFree(alloc_handle);
                Err(result)
            }
            else {
                Ok(())
            }
        }
    }
}

#[inline(always)]
///Determines whenever provided clipboard format is available on clipboard or not.
pub fn is_format_avail(format: u32) -> bool {
    unsafe { IsClipboardFormatAvailable(format) != 0 }
}

#[inline]
///Retrieves number of currently available formats on clipboard.
pub fn count_formats() -> io::Result<i32> {
    let result = unsafe { CountClipboardFormats() };

    if result == 0 {
        let error = io::Error::last_os_error();

        if let Some(raw_error) = error.raw_os_error() {
            if raw_error != 0 {
                return Err(error)
            }
        }
    }

    Ok(result)
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
        }
        else {
            Some(self.idx)
        }
    }

    /// Relies on `count_formats` so it is only reliable
    /// when hinting size for enumeration of all formats.
    ///
    /// Doesn't require opened clipboard.
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, count_formats().ok().map(|val| val as usize))
    }
}

macro_rules! match_format_name {
    ( $name:expr, $( $f:ident ),* ) => {
        match $name {
            $( formats::$f => Some(stringify!($f).to_string()),)*
            formats::CF_GDIOBJFIRST ..= formats::CF_GDIOBJLAST => Some(format!("CF_GDIOBJ{}", $name - formats::CF_GDIOBJFIRST)),
            formats::CF_PRIVATEFIRST ..= formats::CF_PRIVATELAST => Some(format!("CF_PRIVATE{}", $name - formats::CF_PRIVATEFIRST)),
            _ => {
                let format_buff = [0u16; 52];
                unsafe {
                    let buff_p = format_buff.as_ptr() as *mut u16;

                    if GetClipboardFormatNameW($name, buff_p, format_buff.len() as c_int) == 0 {
                        None
                    } else {
                        Some(String::from_utf16_lossy(&format_buff))
                    }
                }
            }
        }
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
pub fn format_name(format: u32) -> Option<String> {
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
                       CF_UNICODETEXT)
}

///Registers a new clipboard format with specified name.
///
///# Returns:
///
///Newly registered format identifier.
///
///# Note:
///
///Custom format identifier is in range `0xC000...0xFFFF`.
pub fn register_format<T: ?Sized + AsRef<std::ffi::OsStr>>(name: &T) -> io::Result<u32> {
    let mut utf16_buff: Vec<u16> = name.as_ref().encode_wide().collect();
    utf16_buff.push(0);

    match unsafe { RegisterClipboardFormatW(utf16_buff.as_ptr()) } {
        0 => Err(io::Error::last_os_error()),
        result => Ok(result),
    }
}
