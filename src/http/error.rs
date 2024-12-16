use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpError {
    #[error(transparent)]
    IO(#[from] tokio::io::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("Download failed with status code: {0}")]
    Download(String)
}
