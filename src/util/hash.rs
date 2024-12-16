use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use sha1::Digest;
use sha1::Sha1;

use super::error::UtilError;

pub fn calculate_sha1<P: AsRef<Path>>(path: P) -> Result<String, UtilError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);
    Ok(format!("{:x}", hasher.finalize()))
}
