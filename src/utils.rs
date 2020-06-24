use core::ptr;

use smart_ptr::unique;
use winapi::ctypes::c_void;

const GHND: winapi::ctypes::c_uint = 0x42;

#[inline]
fn unlock_data(data: *mut u8) {
    unsafe {
        winapi::um::winbase::GlobalUnlock(data as _);
    }
}

#[inline]
fn free_mem(data: *mut u8) {
    unsafe {
        winapi::um::winbase::GlobalFree(data as _);
    }
}

pub type WinMem = unique::Fn<c_void>;

pub trait WinMemImpl: Sized {
    fn new_global_mem(size: usize) -> Option<Self>;
}

pub trait WinMemLock: Sized {
    fn with_lock<R, F: Fn(ptr::NonNull<c_void>) -> R>(&self, cb: F) -> Option<R>;
}

impl WinMemImpl for unique::Fn<c_void> {
    #[inline(always)]
    fn new_global_mem(size: usize) -> Option<Self> {
        unsafe { unique::Fn::from_ptr(winapi::um::winbase::GlobalAlloc(GHND, size as _), free_mem) }
    }
}

impl<D: smart_ptr::Deleter> WinMemLock for unique::Unique<c_void, D> {
    #[inline(always)]
    ///Executes provided callable with locked global memory, when lock is possible
    fn with_lock<R, F: FnMut(ptr::NonNull<c_void>) -> R>(&self, mut cb: F) -> Option<R> {
        let ptr = unsafe {
            winapi::um::winbase::GlobalLock(self.get())
        };
        match ptr::NonNull::new(ptr) {
            Some(ptr) => {
                //Not exactly unique ptr by this point, but then again GlobalAlloc doesn't return
                //memory per se, as we have to get it via GlobalLock
                let _lock = unsafe {
                    WinMem::from_ptr_unchecked(self.get(), unlock_data);
                };
                Some(cb(ptr))
            },
            None => None
        }
    }
}
