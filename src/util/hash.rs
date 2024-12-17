use sha1::{Digest, Sha1};
use std::{fs::File, io::Read, path::Path};

pub fn calculate_sha1<P: AsRef<Path>>(path: P) -> crate::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);
    Ok(format!("{:x}", hasher.finalize()))
}
