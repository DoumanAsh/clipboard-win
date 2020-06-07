//! Image module

use std::io::{self, Cursor, Read, Write};
use core::{mem, ptr, slice};
use core::ops::{Deref, DerefMut};
use core::ffi::c_void;

use winapi::{
    shared::windef::HDC,
    um::{
        minwinbase::LPTR,
        winbase::{LocalAlloc, LocalFree},
        wingdi::{
            CreateDIBitmap, GetDIBits, GetObjectW, BITMAP, BITMAPFILEHEADER, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, CBM_INIT, DIB_RGB_COLORS, RGBQUAD,
        },
        winuser::{EmptyClipboard, GetDC, ReleaseDC, SetClipboardData, CF_BITMAP},
    },
};

struct Dc(HDC);
impl Dc {
    fn new() -> Self {
        Self(unsafe { GetDC(ptr::null_mut()) })
    }
}
impl Drop for Dc {
    fn drop(&mut self) {
        unsafe { ReleaseDC(ptr::null_mut(), self.0) };
    }
}

struct LocalMemory<T>(ptr::NonNull<T>);
impl<T> LocalMemory<T> {
    pub fn new(size: usize) -> Option<Self> {
        let ptr = unsafe { LocalAlloc(LPTR, size) } as *mut _;
        Some(Self(ptr::NonNull::new(ptr)?))
    }

    pub fn as_ptr(&mut self) -> *mut T {
        self.0.as_ptr()
    }
}
impl<T> Drop for LocalMemory<T> {
    fn drop(&mut self) {
        unsafe { LocalFree(self.0.as_ptr() as _) };
    }
}
impl<T> Deref for LocalMemory<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}
impl<T> DerefMut for LocalMemory<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

///Bitmap image from clipboard
pub struct Image {
    ///Raw image data.
    ///These are the bytes that are read when loading a `.bmp` file into memory
    pub bytes: Vec<u8>,
}

impl Image {
    pub(crate) fn from_handle(handle: ptr::NonNull<c_void>) -> io::Result<Self> {
        let mut bitmap = BITMAP {
            bmType: 0,
            bmWidth: 0,
            bmHeight: 0,
            bmWidthBytes: 0,
            bmPlanes: 0,
            bmBitsPixel: 0,
            bmBits: ptr::null_mut(),
        };

        if unsafe {
            GetObjectW(
                handle.as_ptr(),
                mem::size_of::<BITMAP>() as _,
                &mut bitmap as *mut BITMAP as _,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }

        let clr_bits = bitmap.bmPlanes * bitmap.bmBitsPixel;
        let clr_bits = if clr_bits == 1 {
            1
        } else if clr_bits <= 4 {
            4
        } else if clr_bits <= 8 {
            8
        } else if clr_bits <= 16 {
            16
        } else if clr_bits <= 24 {
            24
        } else {
            32
        };

        let info: Option<LocalMemory<BITMAPINFO>> = if clr_bits < 24 {
            LocalMemory::new(
                mem::size_of::<BITMAPINFOHEADER>() + mem::size_of::<RGBQUAD>() * (1 << clr_bits),
            )
        } else {
            LocalMemory::new(mem::size_of::<BITMAPINFOHEADER>())
        };
        let mut info = match info {
            Some(info) => info,
            None => return Err(io::Error::last_os_error()),
        };

        info.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as _;
        info.bmiHeader.biWidth = bitmap.bmWidth;
        info.bmiHeader.biHeight = bitmap.bmHeight;
        info.bmiHeader.biPlanes = bitmap.bmPlanes;
        info.bmiHeader.biBitCount = bitmap.bmBitsPixel;
        info.bmiHeader.biCompression = BI_RGB;
        if clr_bits < 24 {
            info.bmiHeader.biClrUsed = 1 << clr_bits;
        }

        info.bmiHeader.biSizeImage =
            ((((info.bmiHeader.biWidth * clr_bits + 31) & !31) / 8) * info.bmiHeader.biHeight) as _;
        info.bmiHeader.biClrImportant = 0;

        let dc = Dc::new();
        let mut buf = Vec::with_capacity(info.bmiHeader.biSizeImage as _);
        buf.resize(buf.capacity(), 0);

        if unsafe {
            GetDIBits(
                dc.0,
                handle.as_ptr() as _,
                0,
                info.bmiHeader.biHeight as _,
                buf.as_mut_ptr() as _,
                info.as_ptr(),
                DIB_RGB_COLORS,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }

        let mut stream = Vec::new();
        stream.extend_from_slice(&u16::to_le_bytes(0x4d42));
        stream.extend_from_slice(&u32::to_le_bytes(
            mem::size_of::<BITMAPFILEHEADER>() as u32
                + info.bmiHeader.biSize
                + info.bmiHeader.biClrUsed * mem::size_of::<RGBQUAD>() as u32
                + info.bmiHeader.biSizeImage,
        ));
        stream.extend_from_slice(&u16::to_le_bytes(0));
        stream.extend_from_slice(&u16::to_le_bytes(0));
        stream.extend_from_slice(&u32::to_le_bytes(
            mem::size_of::<BITMAPFILEHEADER>() as u32
                + info.bmiHeader.biSize
                + info.bmiHeader.biClrUsed * mem::size_of::<RGBQUAD>() as u32,
        ));

        let h = &info.bmiHeader;
        stream.extend_from_slice(&h.biSize.to_le_bytes());
        stream.extend_from_slice(&h.biWidth.to_le_bytes());
        stream.extend_from_slice(&h.biHeight.to_le_bytes());
        stream.extend_from_slice(&h.biPlanes.to_le_bytes());
        stream.extend_from_slice(&h.biBitCount.to_le_bytes());
        stream.extend_from_slice(&h.biCompression.to_le_bytes());
        stream.extend_from_slice(&h.biSizeImage.to_le_bytes());
        stream.extend_from_slice(&h.biXPelsPerMeter.to_le_bytes());
        stream.extend_from_slice(&h.biYPelsPerMeter.to_le_bytes());
        stream.extend_from_slice(&h.biClrUsed.to_le_bytes());
        stream.extend_from_slice(&h.biClrImportant.to_le_bytes());

        let colors = unsafe {
            slice::from_raw_parts(info.bmiColors.as_ptr(), info.bmiHeader.biClrUsed as _)
        };
        for color in colors {
            stream.push(color.rgbBlue);
            stream.push(color.rgbGreen);
            stream.push(color.rgbRed);
            stream.push(color.rgbReserved);
        }

        stream.write(&buf[..])?;

        Ok(Self { bytes: stream })
    }

    pub(crate) fn write_to_clipboard(&self) -> io::Result<()> {
        fn read_u16<R>(stream: &mut R) -> io::Result<u16>
        where
            R: Read,
        {
            let mut buf = [0 as u8; 2];
            stream.read_exact(&mut buf)?;
            Ok(u16::from_le_bytes(buf))
        }

        fn read_u32<R>(stream: &mut R) -> io::Result<u32>
        where
            R: Read,
        {
            let mut buf = [0 as u8; 4];
            stream.read_exact(&mut buf)?;
            Ok(u32::from_le_bytes(buf))
        }

        fn read_i32<R>(stream: &mut R) -> io::Result<i32>
        where
            R: Read,
        {
            let mut buf = [0 as u8; 4];
            stream.read_exact(&mut buf)?;
            Ok(i32::from_le_bytes(buf))
        }

        let mut stream = Cursor::new(&self.bytes);
        let file_header = BITMAPFILEHEADER {
            bfType: read_u16(&mut stream)?,
            bfSize: read_u32(&mut stream)?,
            bfReserved1: read_u16(&mut stream)?,
            bfReserved2: read_u16(&mut stream)?,
            bfOffBits: read_u32(&mut stream)?,
        };

        let info_header = BITMAPINFOHEADER {
            biSize: read_u32(&mut stream)?,
            biWidth: read_i32(&mut stream)?,
            biHeight: read_i32(&mut stream)?,
            biPlanes: read_u16(&mut stream)?,
            biBitCount: read_u16(&mut stream)?,
            biCompression: read_u32(&mut stream)?,
            biSizeImage: read_u32(&mut stream)?,
            biXPelsPerMeter: read_i32(&mut stream)?,
            biYPelsPerMeter: read_i32(&mut stream)?,
            biClrUsed: read_u32(&mut stream)?,
            biClrImportant: read_u32(&mut stream)?,
        };

        let info = &info_header as *const _ as *const BITMAPINFO;
        let bitmap = &self.bytes[file_header.bfOffBits as _..];

        unsafe {
            let dc = Dc::new();
            let handle = CreateDIBitmap(
                dc.0,
                &info_header as _,
                CBM_INIT,
                bitmap.as_ptr() as _,
                info,
                DIB_RGB_COLORS,
            );
            EmptyClipboard();
            if SetClipboardData(CF_BITMAP, handle as _).is_null() {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(())
    }
}
