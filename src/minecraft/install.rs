use std::{
    any::type_name,
    collections::HashMap,
    env::consts::{ARCH, OS},
    fs,
    path::{Path, PathBuf, MAIN_SEPARATOR_STR},
    sync::Arc,
};

use rayon::iter::ParallelIterator;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};
use reqwest::IntoUrl;
use serde_json::Value;
use tokio::{
    fs::{create_dir_all, rename},
    sync::{mpsc, Mutex, Semaphore},
};

use event_emitter_rs::EventEmitter;

use crate::{
    http::{
        downloader::{download, download_multiple},
        fetch::fetch,
    },
    json::{
        java::{self, JavaFileManifest, JavaManifest},
        version::{
            asset_index::{AssetIndex, File},
            manifest::VersionManifest,
            meta::vanilla::{self, JavaVersion, VersionMeta},
        },
    },
    util::{
        extract::unzip_file,
        hash::calculate_sha1,
        json::{read_json, write_json},
    },
};

use super::{error::MinecraftError, launch::Config, loaders::Loader, version::ParseRule};

pub const VERSION_MANIFEST_ENDPOINT: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

pub const RESOURCES_ENDPOINT: &str = "https://resources.download.minecraft.net";
pub const JAVA_MANIFEST_ENDPOINT: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

#[derive(Clone)]
enum FileType {
    Asset { is_virtual: bool, is_map: bool },
    Library,
    Java,
}

#[derive(Clone)]
struct DownloadFile {
    file_name: String,
    sha1: String,
    url: String,
    path: PathBuf,
    r#type: FileType,
}

pub async fn install<T: Loader>(
    config: &Config<T>,
    emitter: Option<&Arc<Mutex<EventEmitter>>>,
) -> Result<(), MinecraftError> {
    let manifest: VersionManifest = fetch(VERSION_MANIFEST_ENDPOINT).await?;

    let version_name = config
        .version_name
        .clone()
        .unwrap_or(if config.loader.is_some() {
            let name = format!(
                "{}-{}",
                type_name::<T>().split("::").last().unwrap_or("Custom"),
                config.version
            );
            name
        } else {
            config.version.to_string()
        });

    let version_path = config.game_dir.join("versions").join(&version_name);
    let version_jar_path = version_path.join(format!("{}.jar", &version_name));
    let version_json_path = version_path.join(format!("{}.json", &version_name));

    let meta: VersionMeta = if !version_json_path.exists() {
        let mut meta = fetch(
            &manifest
                .versions
                .iter()
                .find(|version| version.id.eq(&config.version))
                .ok_or(MinecraftError::UnknownVersion("Vanilla".to_string()))?
                .url,
        )
        .await?;

        if let Some(loader) = &config.loader {
            meta = loader.merge(&config.game_dir, meta).await?;
        }

        write_json(version_json_path, &meta).await?;
        meta
    } else {
        read_json(version_json_path).await?
    };

    let assets_path = config.game_dir.join("assets");
    let index_path = assets_path.join("indexes");
    let asset_index_path = &index_path.join(format!("{}.json", &meta.asset_index.id));
    let asset_index: AssetIndex = if !asset_index_path.exists() {
        let asset_index = fetch(&meta.asset_index.url).await?;
        write_json(asset_index_path, &asset_index).await?;
        asset_index
    } else {
        read_json(asset_index_path).await?
    };

    if !version_jar_path.exists()
        || !calculate_sha1(&version_jar_path)?.eq(&meta.downloads.client.sha1)
    {
        download(&meta.downloads.client.url, version_jar_path, emitter).await?;
    }

    let natives_path = config.game_dir.join("natives").join(&config.version);

    if !natives_path.is_dir() {
        create_dir_all(&natives_path).await?;
    }

    let check_natives: bool = fs::read_dir(&natives_path)?.count() == 0;

    let mut to_be_extracted: Vec<vanilla::File> = Vec::with_capacity(10);

    let java_version = meta.java_version.unwrap_or(JavaVersion {
        component: "jre-legacy".to_string(),
        major_version: 0,
    });
    let runtime_path = config
        .runtime_dir
        .clone()
        .unwrap_or(config.game_dir.join("runtime"))
        .join(&java_version.component);
    let java_manifest: JavaManifest = fetch(JAVA_MANIFEST_ENDPOINT).await?;

    fn get_java_os() -> String {
        let os = if OS == "macos" { "mac-os" } else { OS };

        let arch: String = match ARCH {
            "x86" => {
                if os == "linux" {
                    "i386".to_string()
                } else {
                    "x86".to_string()
                }
            }
            "x86_64" => "x64".to_string(),
            "aarch64" => "arm64".to_string(),
            _ => panic!("Unsupported architecture"),
        };

        format!("{}-{}", os, arch)
    }

    let java_url = &java_manifest
        .get(&get_java_os())
        .ok_or(MinecraftError::NotFound(
            "Java map by operating system".to_string(),
        ))?
        .get(&java_version.component)
        .ok_or(MinecraftError::UnknownVersion("Java version".to_string()))?
        .first()
        .ok_or(MinecraftError::NotFound("Java gamecore".to_string()))?
        .manifest
        .url;

    let java_files: JavaFileManifest = fetch(java_url).await?;

    let file_map: Vec<DownloadFile> = [
        asset_index
            .objects
            .iter()
            .map(|(key, meta)| {
                let hash = &meta.hash;
                DownloadFile {
                    file_name: key.clone(),
                    sha1: hash.clone(),
                    url: format!("{}/{}/{}", RESOURCES_ENDPOINT, &hash[0..2], hash),
                    path: assets_path.join("objects").join(&hash[0..2]).join(hash),
                    r#type: FileType::Asset {
                        is_map: asset_index.map_to_resources.unwrap_or_default(),
                        is_virtual: asset_index.r#virtual.unwrap_or_default(),
                    },
                }
            })
            .collect::<Vec<_>>(),
        meta.libraries
            .iter()
            .filter_map(|lib| {
                if !lib.rules.parse_rule() {
                    return None;
                }

                let downloads = lib.downloads.as_ref()?;

                if check_natives {
                    if let Some(classifiers) = &downloads.classifiers {
                        let classifier = match OS {
                            "windows" => &classifiers.natives_windows,
                            "linux" => &classifiers.natives_linux,
                            "macos" => &classifiers.natives_osx,
                            _ => panic!("Unknown operating system!"),
                        };

                        if let Some(classifier) = classifier {
                            if let Some(classifier_path) = &classifier.path {
                                let path = config
                                    .game_dir
                                    .join("libraries")
                                    .join(classifier_path.replace("/", MAIN_SEPARATOR_STR));
                                let url = classifier.url.clone();
                                let sha1 = classifier.sha1.clone();
                                to_be_extracted.push(vanilla::File {
                                    path: Some(path.to_string_lossy().into_owned()),
                                    sha1: sha1.clone(),
                                    size: classifier.size,
                                    url: url.clone(),
                                });

                                return Some(DownloadFile {
                                    file_name: PathBuf::from(url.clone())
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string(),
                                    sha1,
                                    url,
                                    path,
                                    r#type: FileType::Library,
                                });
                            }
                        }
                    }
                }

                let artifact = downloads.artifact.as_ref()?;
                Some(DownloadFile {
                    file_name: PathBuf::from(artifact.url.clone())
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    sha1: artifact.sha1.clone(),
                    url: artifact.url.clone(),
                    path: config
                        .game_dir
                        .join("libraries")
                        .join(artifact.path.as_ref()?.replace("/", MAIN_SEPARATOR_STR)),
                    r#type: FileType::Library,
                })
            })
            .collect::<Vec<_>>(),
        java_files
            .files
            .iter()
            .filter_map(|(name, file)| {
                let path = runtime_path.join(name.replace("/", MAIN_SEPARATOR_STR));
                if let Some(downloads) = &file.downloads {
                    return Some(DownloadFile {
                        file_name: name
                            .split(MAIN_SEPARATOR_STR)
                            .last()
                            .unwrap_or(name)
                            .to_string(),
                        path,
                        sha1: downloads.raw.sha1.clone(),
                        url: downloads.raw.url.clone(),
                        r#type: FileType::Java,
                    });
                }
                None
            })
            .collect::<Vec<_>>(),
    ]
    .concat();

    download_necessary(
        file_map,
        &config.game_dir,
        asset_index.map_to_resources.unwrap_or_default()
            || asset_index.r#virtual.unwrap_or_default(),
        emitter,
    )
    .await?;

    if !to_be_extracted.is_empty() {
        create_dir_all(&natives_path).await?;
        for extract in to_be_extracted {
            let path = PathBuf::from(extract.path.unwrap());
            download(&extract.url, &path, emitter).await?;
            unzip_file(&path, &natives_path).await?;
        }
    }

    Ok(())
}

async fn download_necessary(
    files: Vec<DownloadFile>,
    game_dir: &Path,
    legacy: bool,
    emitter: Option<&Arc<Mutex<EventEmitter>>>,
) -> Result<(), MinecraftError> {
    let broken_ones: Vec<(String, PathBuf)> = files
        .par_iter()
        .filter_map(|fix_file| {
            if !fix_file.path.exists() {
                return Some((fix_file.url.clone(), fix_file.path.clone()));
            } else if fix_file.sha1.is_empty() {
                println!("Skipping file hash check since hash is empty.");
                return None;
            } else if let Ok(hash) = calculate_sha1(&fix_file.path) {
                if hash != fix_file.sha1 {
                    return Some((fix_file.url.clone(), fix_file.path.clone()));
                }
            }
            None
        })
        .collect();

    download_multiple(broken_ones, emitter).await?;

    if legacy {
        files.par_iter().for_each(|fix_file| {
            if let FileType::Asset { is_virtual, is_map } = fix_file.r#type {
                let target_path = if is_virtual {
                    game_dir
                        .join("assets")
                        .join("virtual")
                        .join("legacy")
                        .join(&fix_file.file_name)
                } else if is_map {
                    game_dir.join("resources").join(&fix_file.file_name)
                } else {
                    return;
                };

                if let Some(parent) = target_path.parent() {
                    if !parent.is_dir() {
                        fs::create_dir_all(parent).ok();
                    }
                }

                if !target_path.exists() {
                    fs::copy(&fix_file.path, &target_path).ok();
                }
                if let Ok(hash) = calculate_sha1(&target_path) {
                    if hash != fix_file.sha1 {
                        fs::copy(&fix_file.path, target_path).ok();
                    }
                }
            }
        });
    }

    Ok(())
}
