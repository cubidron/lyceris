use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilError {
    #[error("Operation failed after trying {1} times. Reason: {0}")]
    Retry(String, u8),
    #[error(transparent)]
    IO(#[from] tokio::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError)
}
