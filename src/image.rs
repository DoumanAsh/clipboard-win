//! Image module

use winapi::um::wingdi::GetObjectW;

use std::{io, mem};
use std::os::raw::{c_void, c_int};

use crate::utils;

///Bitmap image from clipboard
pub struct Bitmap {
    inner: winapi::shared::windef::HBITMAP,
    data: winapi::um::wingdi::BITMAP,
}

impl Bitmap {
    ///Creates instance from BITMAP handle
    pub fn new(ptr: *mut c_void) -> io::Result<Self> {
        use winapi::um::wingdi::BITMAP;

        let mut data: BITMAP = unsafe { mem::zeroed() };
        let data_ptr = &mut data as *mut BITMAP as *mut winapi::ctypes::c_void;

        match unsafe { GetObjectW(ptr as *mut winapi::ctypes::c_void, mem::size_of::<BITMAP>() as c_int, data_ptr) } {
            0 => Err(utils::get_last_error()),
            _ => Ok(Self {
                inner: ptr as winapi::shared::windef::HBITMAP,
                data
            }),
        }
    }

    #[inline]
    ///Returns raw handle.
    pub fn as_raw(&self) -> winapi::shared::windef::HBITMAP {
        self.inner
    }

    #[inline]
    ///Returns image dimensions as `(width, height)`
    pub fn dimensions(&self) -> (usize, usize) {
        (self.data.bmWidth as usize, self.data.bmHeight as usize)
    }
}
