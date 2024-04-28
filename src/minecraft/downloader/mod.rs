use std::{
    collections::HashMap,
    path::{PathBuf, MAIN_SEPARATOR_STR},
};

use serde::Deserialize;
use tokio::fs;

use crate::{
    minecraft::java::{detect_java_by_cmd, get_manifest_by_version}, network::download_retry, prelude::{Result, R}, reporter::Reporter, utils::{extract_zip, hash_file, hash_file_sha256, hash_files, json_from_file}
};

use super::{
    custom::CustomPackage, serde::{Action, Index, Library, Name}, Case, Launcher, LAUNCHER_API
};

pub struct Downloader<'a> {
    pub launcher: &'a Launcher,
}

pub struct File {
    pub hash: String,
    pub url: String,
    pub state: bool,
    pub path: PathBuf,
}

#[derive(Deserialize,Clone)]
pub struct ServerFile {
    pub name: String,
    pub path: String,
    pub sha256: String,
}

impl<'a> Downloader<'a> {
    pub fn new(launcher: &'a Launcher) -> Downloader {
        Self { launcher }
    }

    pub async fn download_assets(&self) -> Result<()> {
        R.send(Case::SetMessage(
            "Kaynak dosyaları kontrol ediliyor".to_string(),
        ));

        let index = &self.launcher.config.index.clone().unwrap();

        let mut files: Vec<File> = vec![];

        for (key, object) in &index.objects {
            let sub_hash = &object.hash[0..2];
            let hash_path = if index.virtual_.is_some() {
                self.launcher
                    .root_path
                    .join("assets")
                    .join("virtual")
                    .join("legacy")
                    .join(key.replace('/', std::path::MAIN_SEPARATOR_STR))
            } else {
                self.launcher
                    .root_path
                    .join("assets")
                    .join("objects")
                    .join(sub_hash)
                    .join(&object.hash)
            };
            if hash_path.is_file() {
                files.push(File {
                    hash: object.hash.clone(),
                    url: format!(
                        "https://resources.download.minecraft.net/{sub_hash}/{}",
                        object.hash
                    ),
                    state: false,
                    path: hash_path,
                });
            } else {
                R.send(Case::SetMessage(
                    "Eksik kaynak dosyaları yükleniyor".to_string(),
                ));
                download_retry(
                    format!(
                        "https://resources.download.minecraft.net/{sub_hash}/{}",
                        object.hash
                    ),
                    &hash_path
                )
                .await?;
                R.send(Case::AddProgress(1.0));
            }
        }

        let hashes = hash_files(files)?;

        for file in hashes {
            if !file.state {
                download_retry(&file.url, &file.path).await;
            }
            R.send(Case::AddProgress(1.0));
        }

        Ok(())
    }

    pub async fn download_client(&self) -> Result<()> {
        R
            .send(Case::SetMessage("İstemci kontrol ediliyor".to_string()));
        let file_path = self
            .launcher
            .root_path
            .join("versions")
            .join(&self.launcher.version_name)
            .join(format!("{}.jar", self.launcher.version_name));

        if file_path.is_file()
            && hash_file(&file_path)? == self.launcher.config.package.downloads.client.sha1
        {
            R.send(Case::AddProgress(1.0));
            return Ok(());
        }

        R
            .send(Case::SetMessage("İstemci yükleniyor".to_string()));

            download_retry(
            self.launcher.config.package.downloads.client.url.clone(),
            &file_path,
        )
        .await?;

        if self.launcher.config.custom.is_none() {
            fs::write(
                self.launcher
                    .root_path
                    .join("versions")
                    .join(&self.launcher.version_name)
                    .join(format!("{}.json", self.launcher.version_name)),
                serde_json::to_string_pretty(&self.launcher.config.package).unwrap(),
            )
            .await?;
        }

        // ADD ELSE

        Ok(())
    }

    pub async fn download_libraries(&self) -> Result<()> {
        R.send(Case::SetMessage(
            "Kütüphaneler kontrol ediliyor".to_string(),
        ));

        for lib in &self.launcher.config.package.libraries {
            if let Some(artifact) = &lib.downloads.artifact {
                let file_path = self
                    .launcher
                    .root_path
                    .join("libraries")
                    .join(artifact.path.replace('/', MAIN_SEPARATOR_STR));

                if !self.launcher.parse_rule(lib)
                    && (!file_path.is_file() || hash_file(&file_path)? != artifact.sha1)
                {
                    R.send(Case::SetMessage(
                        "Eksik kütüphaneler yükleniyor".to_string(),
                    ));
                    download_retry(&artifact.url, &file_path).await?;
                }
            }
            R.send(Case::AddProgress(1.0));
        }

        if let Some(package) = &self.launcher.config.custom {
            match package {
                CustomPackage::Fabric(package) => {
                    R.send(Case::SetMessage(
                        "Fabric dosyaları kontrol ediliyor".to_string(),
                    ));
                    let mut progress = 0f64;
                    for i in &package.libraries {
                        let parts = i.name.split(':').collect::<Vec<&str>>();
                        let file_name = format!("{}-{}.jar", parts[1], parts[2]);
                        let url = format!(
                            "{}{}/{}/{}/{}",
                            i.url,
                            parts[0].replace('.', "/"),
                            parts[1],
                            parts[2],
                            file_name
                        );
                        let path = self
                            .launcher
                            .root_path
                            .join("libraries")
                            .join(parts[0].replace('.', std::path::MAIN_SEPARATOR_STR))
                            .join(parts[1])
                            .join(parts[2])
                            .join(&file_name);

                        if !path.is_file() {
                            R.send(Case::SetMessage(
                                "Eksik fabric dosyaları yükleniyor".to_string(),
                            ));
                            download_retry(&url, &path).await?;
                        } else if let Some(sha1) = &i.sha1 {
                            if &hash_file(&path)? != sha1 {
                                download_retry(&url, &path).await?;
                            }
                        }
                        R.send(Case::AddProgress(1.0));
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn download_natives(&self) -> Result<()> {
        let mut classifier_url = String::new();
        R.send(Case::SetMessage(
            "Native dosyaları kontrol ediliyor".to_string(),
        ));
        let natives_path = self
            .launcher
            .root_path
            .join("natives")
            .join(self.launcher.version.to_string());

        for lib in &self.launcher.config.package.libraries {
            let mut mapping = &None;
            if let Some(classifiers) = &lib.downloads.classifiers {
                if cfg!(target_os = "windows") {
                    if cfg!(target_arch = "x86") {
                        mapping = &classifiers.natives_windows_32;
                    } else if cfg!(target_arch = "x86_64") {
                        mapping = if classifiers.natives_windows_64.is_none() {
                            &classifiers.natives_windows
                        } else {
                            &classifiers.natives_windows_64
                        }
                    }
                } else if cfg!(target_os = "linux") {
                    mapping = &classifiers.natives_linux;
                } else if cfg!(target_os = "macos") {
                    mapping = &classifiers.natives_macos;
                } else {
                    panic!("Unsupported OS");
                }
                if let Some(natives) = mapping {
                    classifier_url = natives.url.clone();
                }
            }
            if !classifier_url.is_empty() {
                fs::create_dir_all(&natives_path);
                let native_file = natives_path.join("native.jar");
                R
                    .send(Case::SetMessage("Native dosyaları yükleniyor".to_string()));
                download_retry(&classifier_url, &native_file).await?;
                extract_zip(&native_file, &natives_path)?;
                fs::remove_file(native_file).await?;
            }
            R.send(Case::AddProgress(1.0));
        }

        Ok(())
    }

    pub async fn download_custom(&self) -> Result<()> {
        R.send(crate::reporter::Case::SetMessage(
            "Özel dosyalar kontrol ediliyor".to_string(),
        ));

        if let Some(files) = &self.launcher.config.server_files{
            for file in files {
                let remote_path = file.path.replace("\\root\\baso-api\\files\\", "");
                let file_path = self.launcher.root_path.join(&remote_path);
                let optional_file_path = self.launcher.root_path.join(format!("{}.deactive",&remote_path));
                let url = format!("{}/api/files/{}", LAUNCHER_API, remote_path);
                if (!file_path.is_file() && !optional_file_path.is_file()) {
                    download_retry(url, &file_path).await?;
                }else if(file_path.is_file() && !hash_file_sha256(&file_path)?.eq(&file.sha256)){
                    download_retry(url, &file_path).await?;
                }else if(optional_file_path.is_file() && !hash_file_sha256(&optional_file_path)?.eq(&file.sha256)){
                    download_retry(url, &file_path).await?;
                }
                R.send(crate::reporter::Case::AddProgress(1.0));
            }
        }
        Ok(())
    }

    pub async fn download_java(&self) -> Result<PathBuf> {
        R.send(Case::SetMessage(
            "Java dosyaları kontrol ediliyor".to_string(),
        ));
        let manifest = get_manifest_by_version(&self.launcher.java_version).await?;
        let java_path = self
            .launcher
            .java_path
            .join(self.launcher.java_version.to_string());
        for (name, file) in manifest.files {
            let path = java_path.join(name);
            if let Some(downloads) = file.downloads {
                if path.is_file() && hash_file(&path)? == downloads.raw.sha1 {
                    R.send(Case::AddProgress(1.0));
                    continue;
                }
                R.send(Case::SetMessage(
                    "Eksik java dosyaları yükleniyor".to_string(),
                ));
                download_retry(&downloads.raw.url, &path).await?;
            }
            R.send(Case::AddProgress(1.0));
        }

        Ok(java_path)
    }
}
