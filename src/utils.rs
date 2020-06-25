use core::{mem, ptr};

use winapi::ctypes::c_void;

use crate::SysResult;

const GHND: winapi::ctypes::c_uint = 0x42;

#[inline]
fn noop(_: *mut c_void) {
}

#[inline]
fn unlock_data(data: *mut c_void) {
    unsafe {
        winapi::um::winbase::GlobalUnlock(data);
    }
}

#[inline]
fn free_mem(data: *mut c_void) {
    unsafe {
        winapi::um::winbase::GlobalFree(data);
    }
}

pub struct Scope<T: Copy>(T, fn(T));

impl<T: Copy> Drop for Scope<T> {
    #[inline(always)]
    fn drop(&mut self) {
        (self.1)(self.0)
    }
}

pub struct WinMem(Scope<*mut c_void>);

impl WinMem {
    #[inline(always)]
    pub fn new_global_mem(size: usize) -> SysResult<Self> {
        unsafe {
            let mem = winapi::um::winbase::GlobalAlloc(GHND, size as _);
            if mem.is_null() {
                Err(error_code::SystemError::last())
            } else {
                Ok(Self(Scope(mem, free_mem)))
            }
        }
    }

    #[inline(always)]
    pub fn from_borrowed(ptr: ptr::NonNull<c_void>) -> Self {
        Self(Scope(ptr.as_ptr(), noop))
    }

    #[inline(always)]
    pub fn get(&self) -> *mut c_void {
        (self.0).0
    }

    #[inline(always)]
    pub fn release(self) {
        mem::forget(self)
    }

    pub fn lock(&self) -> SysResult<(ptr::NonNull<c_void>, Scope<*mut c_void>)> {
        let ptr = unsafe {
            winapi::um::winbase::GlobalLock(self.get())
        };

        match ptr::NonNull::new(ptr) {
            Some(ptr) => Ok((ptr, Scope(self.get(), unlock_data))),
            None => Err(error_code::SystemError::last()),
        }
    }
}
