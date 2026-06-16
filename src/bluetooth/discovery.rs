#[cfg(target_os="linux")]
use crate::linux::bluetooth;

#[cfg(target_os="macos")]
use crate::macos::bluetooth;

#[cfg(target_os="windows")]
use crate::windows::bluetooth;

pub use bluetooth::*;