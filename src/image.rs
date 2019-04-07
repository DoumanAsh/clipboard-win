//! Image module

use winapi::um::wingdi::GetObjectW;

use std::{slice, io, mem};
use std::os::raw::{c_void, c_int, c_long};

use crate::utils;

///Bitmap image from clipboard
pub struct Bitmap {
    inner: winapi::shared::windef::HBITMAP,
    ///Raw BITMAP data
    pub data: winapi::um::wingdi::BITMAP,
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
    ///Calculates the size in bytes for pixel data
    pub fn size(&self) -> usize {
        let color_bits = (self.data.bmPlanes * self.data.bmBitsPixel) as c_long;
        let result = ((self.data.bmWidth * color_bits + 31) / color_bits) * 4 * self.data.bmHeight;
        result as usize
    }

    #[inline]
    ///Retrieves data of underlying Bitmap's data
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.bmBits as *const _, self.size()) }
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

    ///Writes bitmap to IO object
    ///
    ///Returns number of written bytes
    pub fn write<O: io::Write>(&self, _: O) -> io::Result<usize> {
        unimplemented!()
    }
}
