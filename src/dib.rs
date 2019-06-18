//!DIB Image wrapper
use std::os::raw::{c_void};
use std::io;
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

    #[inline]
    ///Retrieves image size, including header.
    pub fn size(&self) -> usize {
        self.bmp.header.size as usize
    }

    ///Writes data into storage.
    ///
    ///Returns size of written, if there is not enough space returns 0.
    pub fn write(&self, storage: &mut [u8]) -> usize {
        let bmp_size = self.size();
        let dib_size = unsafe {
            (*self.bmp.dib).biSize as usize
        };

        if storage.len() < bmp_size {
            return 0;
        }

        let mut storage_ptr = storage.as_mut_ptr();
        storage_ptr = unsafe {
            storage_ptr.copy_from_nonoverlapping(&self.bmp.header as *const BmpHeader as *const u8, mem::size_of::<BmpHeader>());
            storage_ptr.add(mem::size_of::<BmpHeader>())
        };
        storage_ptr = unsafe {
            storage_ptr.copy_from_nonoverlapping(self.bmp.dib as *const u8, dib_size);
            storage_ptr.add(dib_size)
        };
        unsafe {
            let size = (*self.bmp.dib).biSizeImage as usize;
            let dib_ptr = self.bmp.dib as *const u8;
            storage_ptr.copy_from_nonoverlapping(dib_ptr.add(dib_size) as *const u8, size);
        }

        bmp_size
    }

    ///Extracts BMP data as Vec.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.size());
        let data_slice = unsafe {
            slice::from_raw_parts_mut(data.as_mut_ptr(), data.capacity())
        };

        let written = self.write(data_slice);
        unsafe {
            data.set_len(written);
        }

        data
    }
}
