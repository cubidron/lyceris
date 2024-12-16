use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use zip::read::ZipArchive;

use super::error::UtilError;

pub async fn unzip_file<P: AsRef<Path>>(zip_path: &P, output_dir: &P) -> Result<(), UtilError> {
    // Open the ZIP file synchronously
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Create the output directory if it doesn't exist
    tokio::fs::create_dir_all(&output_dir).await?;

    // Iterate through the ZIP file entries
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let output_path = output_dir.as_ref().join(file.name());

        // Create the output file
        if file.name().ends_with('/') {
            // If it's a directory, create it
            tokio::fs::create_dir_all(&output_path).await?;
        } else {
            // If it's a file, write it
            let mut output_file = tokio::fs::File::create(&output_path).await?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?; // Read the file content synchronously
            output_file.write_all(&buffer).await?; // Write the content asynchronously
        }
    }

    Ok(())
}
