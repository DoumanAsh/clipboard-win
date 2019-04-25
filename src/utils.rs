use std::io;

use winapi::ctypes::c_void;

#[inline(always)]
pub fn get_last_error() -> io::Error {
    io::Error::last_os_error()
}

pub(crate) trait WinNullCheckable: Sized + Copy {
    fn if_null_get_last_error(self) -> io::Result<Self> {
        if self.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(self)
        }
    }
    fn if_null_to_error(self, error_gen: impl Fn() -> io::Error) -> io::Result<Self> {
        if self.is_null() {
            Err(error_gen())
        } else {
            Ok(self)
        }
    }
    fn is_null(self) -> bool;
}

impl WinNullCheckable for *mut c_void {
    fn is_null(self) -> bool {
        self.is_null()
    }
}

impl WinNullCheckable for u32 {
    fn is_null(self) -> bool {
        self == 0
    }
}

impl WinNullCheckable for i32 {
    fn is_null(self) -> bool {
        self == 0
    }
}
