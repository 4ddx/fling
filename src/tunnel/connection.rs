#[cfg(target_os = "linux")]
use crate::linux::connection;

#[cfg(target_os = "macos")]
use crate::macos::connection;

#[cfg(target_os = "windows")]
use crate::windows::connection;

pub use connection::*;