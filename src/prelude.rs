use once_cell::sync::Lazy;

use crate::{error::Error, reporter::Reporter};

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(target_os = "windows")]
pub static CLASSPATH_SEPERATOR: &str = ";";

#[cfg(target_os = "linux")]
pub static CLASSPATH_SEPERATOR: &str = ":";

#[cfg(target_os = "macos")]
pub static CLASSPATH_SEPERATOR: &str = ":";
