use thiserror::Error;

use crate::{http::error::HttpError, util::error::UtilError};

#[derive(Error, Debug)]
pub enum MinecraftError {
    #[error(transparent)]
    Util(#[from] UtilError),
    #[error(transparent)]
    IO(#[from] tokio::io::Error),
    #[error(transparent)]
    Http(#[from] HttpError),
    #[error("Unknown {0} version")]
    UnknownVersion(String),
    #[error("{0} Not Found")]
    NotFound(String)
}