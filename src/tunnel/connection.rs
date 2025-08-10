#[cfg(target_os = "linux")]
use crate::linux::connection;

#[cfg(target_os = "macos")]
use crate::macos::connection;

pub use connection::*;