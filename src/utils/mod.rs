use crate::minecraft::downloader;
use crate::prelude::Result;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use serde::de::DeserializeOwned;
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