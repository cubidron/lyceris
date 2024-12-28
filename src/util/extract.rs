use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use async_zip::base::read::seek::ZipFileReader;
use futures::AsyncReadExt;
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::BufReader,
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::error::Error;

pub async fn extract_file(archive: &PathBuf, out_dir: &Path) -> crate::Result<()> {
    let archive = File::open(archive).await?;
    let archive = BufReader::new(archive).compat();
    let mut reader = ZipFileReader::new(archive).await?;
    for index in 0..reader.file().entries().len() {
        let entry = reader
            .file()
            .entries()
            .get(index)
            .ok_or(Error::NotFound("Entry not found".to_string()))?;
        let path = out_dir.join(entry.filename().as_str()?);
        let entry_is_dir = entry.dir()?;

        let mut entry_reader = reader.reader_without_entry(index).await?;

        if entry_is_dir {
            // The directory may have been created if iteration is out of order.
            if !path.exists() {
                create_dir_all(&path).await?;
            }
        } else {
            // Creates parent directories. They may not exist if iteration is out of order
            // or the archive does not contain directory entries.
            let parent = path
                .parent()
                .ok_or(Error::NotFound("Parent not found".to_string()))?;
            if !parent.is_dir() {
                create_dir_all(parent).await?;
            }
            let writer = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .await?;
            futures_lite::io::copy(&mut entry_reader, &mut writer.compat_write()).await?;

            // Closes the file and manipulates its metadata here if you wish to preserve its metadata from the archive.
        }
    }

    Ok(())
}

pub async fn extract_specific_file(
    archive: &PathBuf,
    target_filename: &str,
    out_dir: &Path,
) -> crate::Result<()> {
    let archive = File::open(archive).await?;
    let archive = BufReader::new(archive).compat();
    let mut reader = ZipFileReader::new(archive).await?;

    // Iterate through the entries in the ZIP file
    for index in 0..reader.file().entries().len() {
        let entry = reader
            .file()
            .entries()
            .get(index)
            .ok_or(Error::NotFound("Entry not found".to_string()))?;

        // Check if the current entry matches the target filename
        if entry.filename().as_str().unwrap() == target_filename {
            let path = out_dir.join(entry.filename().as_str()?);
            let entry_is_dir = entry.dir()?;

            let mut entry_reader = reader.reader_without_entry(index).await?;

            if entry_is_dir {
                // If the entry is a directory, create it if it doesn't exist
                if !path.exists() {
                    create_dir_all(&path).await?;
                }
            } else {
                // Create parent directories if they don't exist
                let parent = path
                    .parent()
                    .ok_or(Error::NotFound("Parent not found".to_string()))?;
                if !parent.is_dir() {
                    create_dir_all(parent).await?;
                }

                // Open the file for writing
                let writer = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .await?;

                // Copy the contents of the entry to the file
                futures_lite::io::copy(&mut entry_reader, &mut writer.compat_write()).await?;

                // Optionally, manipulate the file's metadata here if needed
            }
            return Ok(()); // Exit after extracting the target file
        }
    }

    Ok(())
}

pub async fn read_file_from_jar(
    archive: &PathBuf,
    target_filename: &str,
) -> crate::Result<String> {
    let archive = File::open(archive).await?;
    let archive = BufReader::new(archive).compat();
    let mut reader = ZipFileReader::new(archive).await?;

    // Iterate through the entries in the ZIP file
    for index in 0..reader.file().entries().len() {
        let entry = reader
            .file()
            .entries()
            .get(index)
            .ok_or(Error::NotFound("Entry not found".to_string()))?;

        // Check if the current entry matches the target filename
        if entry.filename().as_str().unwrap() == target_filename {
            let mut entry_reader = reader.reader_without_entry(index).await?;
            let mut contents = String::new();

            // Read the contents of the entry into a string
            entry_reader.read_to_string(&mut contents).await?;

            return Ok(contents); // Return the contents of the specific text file
        }
    }

    Err(Error::NotFound("Target file not found".to_string()))
}


pub async fn extract_specific_directory(
    archive: &PathBuf,
    target_directory: &str,
    out_dir: &Path,
) -> crate::Result<()> {
    let archive = File::open(archive).await?;
    let archive = BufReader::new(archive).compat();
    let mut reader = ZipFileReader::new(archive).await?;

    // Iterate through the entries in the ZIP file
    for index in 0..reader.file().entries().len() {
        let entry = reader
            .file()
            .entries()
            .get(index)
            .ok_or(Error::NotFound("Entry not found".to_string()))?;

        // Check if the current entry matches the target directory
        if entry.filename().as_str().unwrap().starts_with(target_directory) {
            let path = out_dir.join(entry.filename().as_str()?);
            let entry_is_dir = entry.dir()?;

            let mut entry_reader = reader.reader_without_entry(index).await?;

            if entry_is_dir {
                // If the entry is a directory, create it if it doesn't exist
                if !path.exists() {
                    create_dir_all(&path).await?;
                }
            } else {
                // Create parent directories if they don't exist
                let parent = path
                    .parent()
                    .ok_or(Error::NotFound("Parent not found".to_string()))?;
                if !parent.is_dir() {
                    create_dir_all(parent).await?;
                }

                // Open the file for writing
                let writer = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .await?;

                // Copy the contents of the entry to the file
                futures_lite::io::copy(&mut entry_reader, &mut writer.compat_write()).await?;

                // Optionally, manipulate the file's metadata here if needed
            }
        }
    }

    Ok(())
}
