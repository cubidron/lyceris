use core::fmt;
use std::io;

use zip::result::ZipError;

#[derive(thiserror::Error,Debug)]
pub enum Error{
    #[error("Network error occured : {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("IO Error occured : {0}")]
    IOError(#[from] io::Error),
    #[error("Serialize error occured : {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Zip extract error occured : {0}")]
    ZipError(#[from] ZipError),
    #[error("Fabric error occured : {0}")]
    FabricError(FabricError),
    #[error("Quilt error occured : {0}")]
    QuiltError(QuiltError)
}

#[derive(Debug)]
pub enum FabricError{
    PackageNotFound
}

#[derive(Debug)]
pub enum QuiltError{
    PackageNotFound
}

impl fmt::Display for FabricError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self{
            FabricError::PackageNotFound=>{
                write!(f,"Fabric package not found")
            }
        }
    }
}

impl fmt::Display for QuiltError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self{
            QuiltError::PackageNotFound=>{
                write!(f,"Quilt package not found")
            }
        }
    }
}