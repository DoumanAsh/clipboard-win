use core::{mem, ptr};

use error_code::ErrorCode;

use crate::{sys, SysResult};
use crate::types::{c_void, c_uint};

const GHND: c_uint = 0x42;

const BYTES_LAYOUT: alloc::alloc::Layout = alloc::alloc::Layout::new::<u8>();

#[cold]
#[inline(never)]
pub fn unlikely_empty_size_result<T: Default>() -> T {
    Default::default()
}

#[cold]
#[inline(never)]
pub fn unlikely_last_error() -> ErrorCode {
    ErrorCode::last_system()
}

#[inline]
fn noop(_: *mut c_void) {
}

#[inline]
fn free_rust_mem(data: *mut c_void) {
    unsafe {
        alloc::alloc::dealloc(data as _, BYTES_LAYOUT)
    }
}

#[inline]
fn unlock_data(data: *mut c_void) {
    unsafe {
        sys::GlobalUnlock(data);
    }
}

#[inline]
fn free_global_mem(data: *mut c_void) {
    unsafe {
        sys::GlobalFree(data);
    }
}

pub struct Scope<T: Copy>(pub T, pub fn(T));

impl<T: Copy> Drop for Scope<T> {
    #[inline(always)]
    fn drop(&mut self) {
        (self.1)(self.0)
    }
}

pub struct RawMem(Scope<*mut c_void>);

impl RawMem {
    #[inline(always)]
    pub fn new_rust_mem(size: usize) -> SysResult<Self> {
        let mem = unsafe {
            alloc::alloc::alloc_zeroed(alloc::alloc::Layout::array::<u8>(size).expect("To create layout for bytes"))
        };

        if mem.is_null() {
            Err(unlikely_last_error())
        } else {
            Ok(Self(Scope(mem as _, free_rust_mem)))
        }
    }

    #[inline(always)]
    pub fn new_global_mem(size: usize) -> SysResult<Self> {
        unsafe {
            let mem = sys::GlobalAlloc(GHND, size as _);
            if mem.is_null() {
                Err(unlikely_last_error())
            } else {
                Ok(Self(Scope(mem, free_global_mem)))
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
            sys::GlobalLock(self.get())
        };

        match ptr::NonNull::new(ptr) {
            Some(ptr) => Ok((ptr, Scope(self.get(), unlock_data))),
            None => Err(ErrorCode::last_system()),
        }
    }
}
