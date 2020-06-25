#![cfg(windows)]
//! This crate provide simple means to operate with Windows clipboard.
//!
//!# Note keeping Clipboard around:
//!
//! In Windows [Clipboard](struct.Clipboard.html) opens globally and only one application can set data onto format at the time.
//!
//! Therefore as soon as operations are finished, user is advised to close [Clipboard](struct.Clipboard.html).
//!
//!# Clipboard
//!
//! All read and write access to Windows clipboard requires user to open it.
//!
//!# Error handling
//!
//!For simplicity sake, errors are not returned in case of failure.
//!Due to the way winapi works, user can use [last_os_error](https://doc.rust-lang.org/std/io/struct.Error.html#method.last_os_error)
//!to find out what's wrong.
//!
//!Alternatively for `no_std` environment you can use [error-code](https://crates.io/crates/error-code)
//!
//!# Usage
//!
//!```
//!use clipboard_win::{Clipboard, formats, Getter, Setter};
//!
//!const SAMPLE: &str = "MY loli sample ^^";
//!
//!let _clip = Clipboard::new_attempts(10).expect("Open clipboard");
//!formats::Unicode.write_clipboard(&SAMPLE).expect("Write sample");
//!
//!let mut output = String::new();
//!
//!assert_eq!(formats::Unicode.read_clipboard(&mut output).expect("Read sample"), SAMPLE.len());
//!assert_eq!(output, SAMPLE);
//!
//!//Efficiently re-use buffer ;)
//!output.clear();
//!assert_eq!(formats::Unicode.read_clipboard(&mut output).expect("Read sample"), SAMPLE.len());
//!assert_eq!(output, SAMPLE);
//!
//!//Or take the same string twice?
//!assert_eq!(formats::Unicode.read_clipboard(&mut output).expect("Read sample"), SAMPLE.len());
//!assert_eq!(format!("{0}{0}", SAMPLE), output);
//!```

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

extern crate alloc;

pub mod formats;
pub mod raw;
pub(crate) mod utils;

pub use raw::{empty, seq_num, size, is_format_avail, EnumFormats};
pub use formats::Unicode;

///Alias to result used by this crate
pub type SysResult<T> = Result<T, error_code::SystemError>;

///Clipboard instance, which allows to perform clipboard ops.
///
///# Note:
///
///You can have only one such instance across your program.
///
///# Warning:
///
///In Windows Clipboard opens globally and only one application can set data
///onto format at the time.
///
///Therefore as soon as operations are finished, user is advised to close Clipboard.
pub struct Clipboard {
    _dummy: ()
}

impl Clipboard {
    #[inline(always)]
    ///Attempts to open clipboard, returning clipboard instance on success.
    pub fn new() -> SysResult<Self> {
        raw::open().map(|_| Self { _dummy: () })
    }

    #[inline]
    ///Attempts to open clipboard, giving it `num` retries in case of failure.
    pub fn new_attempts(mut num: usize) -> SysResult<Self> {
        loop {
            match Self::new() {
                Ok(this) => break Ok(this),
                Err(err) => match num {
                    0 => break Err(err),
                    _ => num -= 1,
                }
            }
        }
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        let _ = raw::close();
    }
}

///Describes format getter, specifying data type as type param
///
///Default implementations only perform write, without opening/closing clipboard
pub trait Getter<Type> {
    ///Reads content of clipboard into `out`, returning number of bytes read on success, or otherwise 0.
    fn read_clipboard(&self, out: &mut Type) -> SysResult<usize>;
}

///Describes format setter, specifying data type as type param
///
///Default implementations only perform write, without opening/closing clipboard
pub trait Setter<Type> {
    ///Writes content of `data` onto clipboard, returning whether it was successful or not
    fn write_clipboard(&self, data: &Type) -> SysResult<()>;
}

#[inline(always)]
///Runs provided callable with open clipboard, returning whether clipboard was open successfully.
///
///If clipboard fails to open, callable is not invoked.
pub fn with_clipboard<F: FnMut()>(mut cb: F) -> SysResult<()> {
    let _clip = Clipboard::new()?;
    cb();
    Ok(())
}

#[inline(always)]
///Runs provided callable with open clipboard, returning whether clipboard was open successfully.
///
///If clipboard fails to open, attempts `num` number of retries before giving up.
///In which case closure is not called
pub fn with_clipboard_attempts<F: FnMut()>(num: usize, mut cb: F) -> SysResult<()> {
    let _clip = Clipboard::new_attempts(num)?;
    cb();
    Ok(())
}
