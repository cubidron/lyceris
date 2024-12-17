use thiserror::Error;
use tokio::time::error::Elapsed;

#[derive(Error, Debug)]
pub enum HttpError {
    #[error(transparent)]
    IO(#[from] tokio::io::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("Download failed with status code: {0}")]
    Download(String),
    #[error("Timeout error")]
    Timeout(#[from] Elapsed),
}
