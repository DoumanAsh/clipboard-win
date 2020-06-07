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

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

extern crate alloc;

pub mod formats;
pub mod raw;
