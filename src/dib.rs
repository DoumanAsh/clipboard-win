//!DIB Image wrapper
use std::os::raw::{c_void};
use std::io::{self, Write};
use core::{slice, mem};

use crate::utils::LockedData;

#[repr(C)]
#[derive(Clone, Copy)]
struct BmpHeader {
    signature: [u8; 2],
    size: u32,
    reserved: u32,
    offset: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Bmp {
    header: BmpHeader,
    dib: *const winapi::um::wingdi::BITMAPINFOHEADER
}

///Represents locked `DIB` clipboard data
pub struct Image {
    bmp: Bmp,
    _guard: LockedData,
}

impl Image {
    ///Constructs new image from clipboard data handle.
    pub fn new(handle: *mut c_void) -> io::Result<Self> {
        let (data, _guard) = LockedData::new(handle)?;

        let dib = data.as_ptr();
        let mut bmp = Bmp {
            header: BmpHeader {
                signature: [0x42, 0x4D],
                size: 0,
                reserved: 0,
                offset: 0
            },
            dib,
        };

        bmp.header.offset = unsafe {
            mem::size_of::<BmpHeader>() as u32 + (*dib).biSize
        };
        bmp.header.size = unsafe {
            (*bmp.dib).biSizeImage + bmp.header.offset
        };

        Ok(Self {
            bmp,
            _guard
        })
    }

    ///Access raw image data
    pub fn as_bytes(&self) -> &[u8] {
        let size = unsafe { winapi::um::winbase::GlobalSize(self._guard.0) };
        let data_ptr = self.bmp.dib as *const u8;

        unsafe {
            slice::from_raw_parts(data_ptr, size)
        }
    }

    #[inline]
    ///Retrieves image size, including header.
    pub fn size(&self) -> usize {
        self.bmp.header.size as usize
    }

    ///Writes data into storage.
    ///
    ///Returns size of written, if there is not enough space returns 0.
    pub fn write<W: Write>(&self, storage: &mut W) -> io::Result<usize> {
        let dib_size = unsafe {
            (*self.bmp.dib).biSize as usize
        };

        let bmp_header = unsafe {
            slice::from_raw_parts(&self.bmp.header as *const BmpHeader as *const u8, mem::size_of::<BmpHeader>())
        };
        storage.write_all(bmp_header)?;

        let dib_header = unsafe {
            slice::from_raw_parts(self.bmp.dib as *const u8, dib_size)
        };
        storage.write_all(dib_header)?;

        let image_data = unsafe {
            let size = (*self.bmp.dib).biSizeImage as usize;
            let dib_ptr = self.bmp.dib as *const u8;
            slice::from_raw_parts(dib_ptr.add(dib_size), size)
        };
        storage.write_all(image_data)?;

        Ok(self.size())
    }

    ///Extracts BMP data as Vec.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.size());
        let _ = self.write(&mut data);

        data
    }
}
