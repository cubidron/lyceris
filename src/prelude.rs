use once_cell::sync::Lazy;

use crate::{error::Error, reporter::Reporter};

pub type Result<T> = std::result::Result<T, Error>;

pub static R: Lazy<ProgressionReporter> = Lazy::new(|| ProgressionReporter {});

/// This reporter will be used for every operation.
///
/// Overriding this is disabled for better usage.
#[derive(Clone)]
pub struct ProgressionReporter {}

impl Reporter for ProgressionReporter {
    fn send(&self, case: crate::reporter::Case) {}
}

#[cfg(target_os = "windows")]
pub static CLASSPATH_SEPERATOR: &str = ";";

#[cfg(target_os = "linux")]
pub static CLASSPATH_SEPERATOR: &str = ":";

#[cfg(target_os = "macos")]
pub static CLASSPATH_SEPERATOR: &str = ":";
