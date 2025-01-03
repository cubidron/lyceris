use std::{
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    task::block_in_place,
};
use zip::ZipArchive;

pub async fn extract_file(jar_path: &PathBuf, output_dir: &PathBuf) -> crate::Result<()> {
    // Read the JAR file into memory asynchronously
    let jar_data = fs::read(jar_path).await?;
    let cursor = Cursor::new(jar_data);

    let mut archive = block_in_place(|| ZipArchive::new(cursor))?;

    // Iterate through the files in the JAR
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let output_path = Path::new(output_dir).join(file.name());

        // Create the output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write the file to the output directory
        let mut output_file = File::create(output_path).await?;

        // Read the file in chunks and write asynchronously
        let mut buffer = vec![0; 4096]; // 4 KB buffer
        loop {
            let n = tokio::task::block_in_place(|| file.read(&mut buffer))?;
            if n == 0 {
                break; // End of file
            }
            output_file.write_all(&buffer[..n]).await?;
        }
    }

    Ok(())
}

pub async fn extract_specific_file(
    jar_path: &PathBuf,
    file_name: &str,
    output_dir: &PathBuf,
) -> crate::Result<()> {
    let jar_data = fs::read(jar_path).await?;
    let cursor = Cursor::new(jar_data);

    let mut archive = block_in_place(|| ZipArchive::new(cursor))?;

    let mut file = archive.by_name(file_name)?;
    let output_path = Path::new(output_dir).join(file_name);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let mut output_file = File::create(output_path).await?;
    let mut buffer = vec![0; 4096];

    loop {
        let n = tokio::task::block_in_place(|| file.read(&mut buffer))?;
        if n == 0 {
            break; // End of file
        }
        output_file.write_all(&buffer[..n]).await?;
    }

    Ok(())
}

pub async fn read_file_from_jar(jar_path: &PathBuf, file_name: &str) -> crate::Result<String> {
    let jar_data = fs::read(jar_path).await?;
    let cursor = Cursor::new(jar_data);

    let mut archive = block_in_place(|| ZipArchive::new(cursor))?;

    let mut file = archive.by_name(file_name)?;

    let mut content = vec![];
    let mut buffer = vec![0; 4096];

    loop {
        let n = tokio::task::block_in_place(|| file.read(&mut buffer))?;
        if n == 0 {
            break; // End of file
        }
        content.extend_from_slice(&buffer[..n]);
    }

    Ok(String::from_utf8(content)?)
}

pub async fn extract_specific_directory(
    jar_path: &PathBuf,
    dir_name: &str,
    output_dir: &Path,
) -> crate::Result<()> {
    let jar_data = fs::read(jar_path).await?;
    let cursor = Cursor::new(jar_data);

    let mut archive = block_in_place(|| ZipArchive::new(cursor))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = file.name();

        // Check if the file path starts with the specified directory name
        if file_path.starts_with(dir_name) {
            let output_path = Path::new(output_dir).join(file_path);

            // Create the output directory if it doesn't exist
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            // Write the file to the output directory
            let mut output_file = File::create(output_path).await?;
            let mut buffer = vec![0; 4096]; // 4 KB buffer

            // Read the file in chunks and write asynchronously
            loop {
                let n = tokio::task::block_in_place(|| file.read(&mut buffer))?;
                if n == 0 {
                    break; // End of file
                }
                output_file.write_all(&buffer[..n]).await?;
            }
        }
    }

    Ok(())
}
