use crate::error;
use crate::minecraft::downloader;
use crate::prelude::Result;
use base64::prelude::*;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use sha1::Digest;
use sha1::Sha1;
use std::collections::HashSet;
use std::fs::read;
use std::fs::read_dir;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};
use zip::ZipArchive;

pub fn json_from_file<T: DeserializeOwned>(file_path: impl AsRef<Path>) -> Result<T> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let json: T = serde_json::from_reader(reader)?;
    Ok(json)
}

pub enum Algorithm {
    SHA1,
}

pub fn hash_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn hash_file_sha256(path: &Path) -> Result<String> {
    let bytes = read(path)?;
    let hash = sha256::digest(bytes);
    Ok(hash)
}

pub fn hash_files(paths: Vec<downloader::File>) -> Result<Vec<downloader::File>> {
    Ok(paths
        .into_par_iter()
        .map(|mut file| {
            if let Ok(hash) = hash_file(&file.path){
                if hash.eq(&file.hash){
                    file.state = true;
                }
            }
            file
        })
        .collect())
}
pub fn extract_zip(
    file_path: impl AsRef<Path>,
    target_path: impl AsRef<Path> + std::convert::AsRef<std::ffi::OsStr>,
) -> Result<()> {
    let file = File::open(file_path)?;

    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = file.mangled_name();

        if file.is_dir() {
            let directory_path = Path::new(&target_path).join(file_path);
            std::fs::create_dir_all(&directory_path)?;
        } else {
            let mut file_buffer = File::create(Path::new(&target_path).join(file_path))?;
            std::io::copy(&mut file, &mut file_buffer)?;
        }
    }
    Ok(())
}

pub fn read_file_from_jar(jar_path: &PathBuf, file_name: &str) -> Result<String> {
    // Open the JAR (ZIP) file
    let file = File::open(jar_path)?;
    let reader = BufReader::new(file);

    // Create a ZipArchive from the reader
    let mut archive = ZipArchive::new(reader)?;

    // Iterate through the files in the archive to find the specific file
    for i in 0..archive.len() {
        let mut zip_file = archive.by_index(i)?;
        if zip_file.name() == file_name {
            // Read the contents of the file
            let mut file_contents = String::new();
            zip_file.read_to_string(&mut file_contents)?;
            return Ok(file_contents);
        }
    }

    Err(format!("File '{}' not found in the JAR", file_name).into())
}

pub fn extract_file_from_jar(jar_path: &PathBuf, file_name: &str, output_path: &PathBuf) -> Result<()> {
    let file = File::open(jar_path)?;
    let reader = BufReader::new(file);

    let mut archive = ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut zip_file = archive.by_index(i)?;
        if zip_file.name() == file_name {
            let output_file_path = Path::new(output_path);
            let mut output_file = File::create(output_file_path)?;

            std::io::copy(&mut zip_file, &mut output_file)?;
            return Ok(());
        }
    }

    Err(format!("File '{}' not found in the JAR", file_name).into())
}

pub fn recurse_files(path: impl AsRef<Path>) -> std::io::Result<HashSet<PathBuf>> {
    let mut buf:HashSet<PathBuf> = HashSet::new();
    let entries = read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;

        if meta.is_dir() {
            let mut subdir = recurse_files(entry.path())?;
            buf.extend(subdir);
        }

        if meta.is_file() {
            buf.insert(entry.path());
        }
    }

    Ok(buf)
}

pub fn decode_base64_url(encoded: &str) -> Result<Vec<u8>> {
    // URL güvenli base64'teki '-' ve '_' karakterlerini düzelt
    let mut base64 = encoded.replace('-', "+").replace('_', "/");
    
    // Doldurma karakterlerini ekle
    let padding = 4 - (base64.len() % 4);
    if padding < 4 {
        base64.push_str(&"=".repeat(padding));
    }
    
    // Base64 çözümleme.
    let decoded = BASE64_URL_SAFE.decode(&base64)?;
    Ok(decoded)
}