//!Clipboard related utilities.

use core::ptr;
use std::io;

///Utility class to automatically call `GlobalUnlock` on `HANDLE`
pub struct LockedData(pub winapi::shared::ntdef::HANDLE);

impl LockedData {
    #[inline]
    ///Locks handle using `GlobalLock` and returns (pointer, guard) on success.
    pub fn new<T>(handle: winapi::shared::ntdef::HANDLE) -> io::Result<(ptr::NonNull<T>, Self)> {
        let ptr = unsafe {
            winapi::um::winbase::GlobalLock(handle)
        };

        match ptr::NonNull::new(ptr as *mut _) {
            Some(ptr) => Ok((ptr, LockedData(handle))),
            None => Err(io::Error::last_os_error()),
        }
    }
}

impl Drop for LockedData {
    fn drop(&mut self) {
        unsafe {
            winapi::um::winbase::GlobalUnlock(self.0);
        }
    }
}

