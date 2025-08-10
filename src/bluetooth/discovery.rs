#[cfg(target_os="linux")]
use crate::linux::bluetooth;

#[cfg(target_os="macos")]
use crate::macos::bluetooth;

pub use bluetooth::*;