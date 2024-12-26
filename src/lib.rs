pub mod auth;
pub mod error;
pub mod http;
pub mod json;
pub mod minecraft;
pub mod util;

use minecraft::{config::Config, emitter::Emitter, install::install, loader::{fabric::Fabric, forge::Forge, quilt::Quilt}};

use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;
