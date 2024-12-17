pub mod auth;
pub mod error;
pub mod http;
pub mod json;
pub mod macros;
pub mod minecraft;
pub mod util;

use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

