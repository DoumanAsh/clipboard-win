//!DIB Image wrapper
use std::os::raw::{c_void};
use std::io::{self, Write};
use core::{slice};

use crate::utils::LockedData;

use lazy_bytes_cast::slice::AsByteSlice;

const HEADER_SIZE: u32 = 14;

#[repr(C)]
#[derive(Clone, Copy)]
struct BmpHeader {
    signature: u16,
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
    pub(crate) fn from_handle(handle: *mut c_void) -> io::Result<Self> {
        let (data, _guard) = LockedData::new(handle)?;

        let dib = data.as_ptr();
        let mut bmp = Bmp {
            header: BmpHeader {
                signature: 0x4D42,
                size: 0,
                reserved: 0,
                offset: 0
            },
            dib,
        };

        bmp.header.offset = unsafe {
            HEADER_SIZE + (*dib).biSize
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
        //BMP Header
        storage.write_all(self.bmp.header.signature.as_slice())?;
        storage.write_all(self.bmp.header.size.as_slice())?;
        storage.write_all(self.bmp.header.reserved.as_slice())?;
        storage.write_all(self.bmp.header.offset.as_slice())?;

        //DIB Header
        let dib = unsafe {
            &*self.bmp.dib
        };
        storage.write_all(dib.biSize.as_slice())?;
        storage.write_all(dib.biWidth.as_slice())?;
        storage.write_all(dib.biHeight.as_slice())?;
        storage.write_all(dib.biPlanes.as_slice())?;
        storage.write_all(dib.biBitCount.as_slice())?;
        storage.write_all(dib.biCompression.as_slice())?;
        storage.write_all(dib.biSizeImage.as_slice())?;
        storage.write_all(dib.biXPelsPerMeter.as_slice())?;
        storage.write_all(dib.biYPelsPerMeter.as_slice())?;
        storage.write_all(dib.biClrUsed.as_slice())?;
        storage.write_all(dib.biClrImportant.as_slice())?;

        //Image data
        let image_data = unsafe {
            let image_ptr = self.bmp.dib.add(1) as *const u8;
            slice::from_raw_parts(image_ptr, dib.biSizeImage as usize)
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
