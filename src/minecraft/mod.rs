pub mod launch;
pub mod version;
pub mod install;
pub mod error;
pub mod loaders;

#[cfg(target_os = "windows")]
pub const TARGET_OS: &str = "windows";
#[cfg(target_os = "macos")]
pub const TARGET_OS: &str = "osx";

#[cfg(target_os = "linux")]
pub const TARGET_OS: &str = "linux";

#[cfg(target_arch = "x86")]
pub const TARGET_ARCH: &str = "x86";

#[cfg(target_arch = "x86_64")]
pub const TARGET_ARCH: &str = "x86_64";

#[cfg(target_arch = "arm")]
pub const NATIVE_ARCH: &str = "arm";

#[cfg(target_arch = "aarch64")]
pub const TARGET_ARCH: &str = "aarch64";

#[cfg(target_arch = "x86")]
pub const NATIVE_ARCH: &str = "32";
#[cfg(target_arch = "x86_64")]
pub const NATIVE_ARCH: &str = "64";

#[cfg(target_arch = "arm")]
pub const NATIVE_ARCH: &str = "64";

#[cfg(target_arch = "aarch64")]
pub const NATIVE_ARCH: &str = "64";

#[cfg(target_os = "windows")]
pub const CLASSPATH_SEPARATOR: &str = ";";

#[cfg(not(target_os = "windows"))]
pub const CLASSPATH_SEPARATOR: &str = ":";