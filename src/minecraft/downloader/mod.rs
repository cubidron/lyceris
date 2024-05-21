use std::{
    fs::{self, read_dir},
    path::{PathBuf, MAIN_SEPARATOR_STR},
};

use async_trait::async_trait;
use futures_util::{future::try_join_all, lock::MutexGuard};
use tokio::try_join;

use crate::{
    network::download_retry,
    prelude::{Result, R},
    reporter::{Case, Progress, Reporter},
    utils::{extract_zip, hash_file, hash_files},
};

use super::{
    java::get_manifest_by_version,
    version::{Custom, MinecraftVersion},
    Cache, Instance, CACHE,
};

pub struct File {
    pub hash: String,
    pub url: String,
    pub state: bool,
    pub path: PathBuf,
}

#[async_trait]
pub trait Downloader {
    async fn download_assets(&self, cache: &MutexGuard<'_, Cache>) -> Result<()>;
    async fn download_client(&self, cache: &MutexGuard<'_, Cache>) -> Result<()>;
    async fn download_libraries(&self, cache: &MutexGuard<'_, Cache>) -> Result<()>;
    async fn download_natives(&self, cache: &MutexGuard<'_, Cache>) -> Result<()>;
    async fn download_java(&self) -> Result<()>;
    async fn install(&self, cache: &MutexGuard<'_, Cache>) -> Result<((), (), (), (), ())> {
        try_join!(
            self.download_client(cache),
            self.download_assets(cache),
            self.download_libraries(cache),
            self.download_natives(cache),
            self.download_java(),
        )
    }
}
#[async_trait]
impl<R: Reporter> Downloader for Instance<R> {
    async fn download_assets(&self, cache: &MutexGuard<'_, Cache>) -> Result<()> {
        R.set_message(t!("resources_check").to_string());

        let index = &cache.index.clone();

        let mut files: Vec<File> = vec![];

        for (key, object) in &index.objects {
            let sub_hash = &object.hash[0..2];
            let hash_path = if index.virtual_.is_some() {
                self.config
                    .root_path
                    .join("assets")
                    .join("virtual")
                    .join("legacy")
                    .join(key.replace('/', std::path::MAIN_SEPARATOR_STR))
            } else {
                self.config
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
                R.set_message(t!("resources_download_missing").to_string());
                download_retry(
                    format!(
                        "https://resources.download.minecraft.net/{sub_hash}/{}",
                        object.hash
                    ),
                    &hash_path,
                    &self.reporter,
                )
                .await?;
                R.add_progress(1.0);
            }
        }
        let files = hash_files(files)?;

        for file in files {
            if !file.state {
                download_retry(&file.url, &file.path, &self.reporter).await?;
            }
            R.add_progress(1.0);
        }

        Ok(())
    }

    async fn download_client(&self, cache: &MutexGuard<'_, Cache>) -> Result<()> {
        R.set_message(t!("client_check").to_string());

        let file_path = if let Some(instance_path) = &self.config.instance_path {
            instance_path
                .join(&self.config.instance_name)
                .join("versions")
                .join(&self.config.version_name)
                .join(format!("{}.jar", self.config.version_name))
        } else {
            self.config
                .root_path
                .join("versions")
                .join(&self.config.version_name)
                .join(format!("{}.jar", self.config.version_name))
        };

        if file_path.is_file() && hash_file(&file_path)? == cache.package.downloads.client.sha1 {
            R.add_progress(1.0);
            return Ok(());
        }

        R.set_message(t!("client_install").to_string());

        download_retry(
            cache.package.downloads.client.url.clone(),
            &file_path,
            &self.reporter,
        )
        .await?;

        if let MinecraftVersion::Custom(_) = self.config.version {
            // fs::write(
            //     self.config
            //         .root_path
            //         .join("versions")
            //         .join(&self.config.version_name)
            //         .join(format!("{}.json", self.config.version_name)),
            //     serde_json::to_string_pretty(&cache.package).unwrap(),
            // )?
        } else {
            fs::write(
                if let Some(instance_path) = &self.config.instance_path {
                    instance_path
                        .join(&self.config.instance_name)
                        .join("versions")
                        .join(&self.config.version_name)
                        .join(format!("{}.json", self.config.version_name))
                } else {
                    self.config
                        .root_path
                        .join("versions")
                        .join(&self.config.version_name)
                        .join(format!("{}.json", self.config.version_name))
                },
                serde_json::to_string_pretty(&cache.package).unwrap(),
            )?
        }

        // ADD ELSE

        Ok(())
    }

    async fn download_libraries(&self, cache: &MutexGuard<'_, Cache>) -> Result<()> {
        R.set_message(t!("libraries_check").to_string());

        for lib in &cache.package.libraries {
            if let Some(artifact) = &lib.downloads.artifact {
                let file_path = self
                    .config
                    .root_path
                    .join("libraries")
                    .join(artifact.path.replace('/', MAIN_SEPARATOR_STR));

                if !self.parse_rule(lib)
                    && (!file_path.is_file() || hash_file(&file_path)? != artifact.sha1)
                {
                    R.set_message(t!("libraries_download_missing").to_string());
                    download_retry(&artifact.url, &file_path, &self.reporter).await?;
                }
            }
            R.add_progress(1.0);
        }

        if let super::version::MinecraftVersion::Custom(ext) = &self.config.version {
            match ext {
                Custom::Fabric(v) => {
                    if let Some(package) = &v.package {
                        R.set_message(t!("fabric_check").to_string());
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
                                .config
                                .root_path
                                .join("libraries")
                                .join(parts[0].replace('.', std::path::MAIN_SEPARATOR_STR))
                                .join(parts[1])
                                .join(parts[2])
                                .join(&file_name);

                            if !path.is_file() {
                                R.set_message(t!("fabric_download_missing").to_string());
                                download_retry(&url, &path, &self.reporter).await?;
                            } else if let Some(sha1) = &i.sha1 {
                                if &hash_file(&path)? != sha1 {
                                    download_retry(&url, &path, &self.reporter).await?;
                                }
                            }
                            R.add_progress(1.0);
                        }
                    }
                }
                Custom::Quilt(v) => {
                    if let Some(package) = &v.package {
                        R.set_message(t!("quilt_check").to_string());
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
                                .config
                                .root_path
                                .join("libraries")
                                .join(parts[0].replace('.', std::path::MAIN_SEPARATOR_STR))
                                .join(parts[1])
                                .join(parts[2])
                                .join(&file_name);

                            if !path.is_file() {
                                R.set_message(t!("quilt_download_missing").to_string());
                                download_retry(&url, &path,&self.reporter).await?;
                            } else if let Some(sha1) = &i.sha1 {
                                if &hash_file(&path)? != sha1 {
                                    download_retry(&url, &path,&self.reporter).await?;
                                }
                            }
                            R.add_progress(1.0);
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }
        Ok(())
    }

    async fn download_java(&self) -> Result<()> {
        R.set_message(t!("java_check").to_string());
        let manifest = get_manifest_by_version(&self.config.java_version).await?;
        let java_path = self
            .config
            .java_path
            .join(self.config.java_version.to_string());
        for (name, file) in manifest.files {
            let path = java_path.join(name);
            if let Some(downloads) = file.downloads {
                if path.is_file() && hash_file(&path)? == downloads.raw.sha1 {
                    R.add_progress(1.0);
                    continue;
                }
                R.set_message(t!("java_download_missing").to_string());
                download_retry(&downloads.raw.url, &path,&self.reporter).await?;
            }
            R.add_progress(1.0);
        }

        Ok(())
    }

    async fn download_natives(&self, cache: &MutexGuard<'_, Cache>) -> Result<()> {
        let mut classifier_url = String::new();

        R.set_message(t!("natives_check").to_string());
        let natives_path = self
            .config
            .root_path
            .join("natives")
            .join(self.config.version.to_string());

        if natives_path.is_dir() {
            println!("{}", fs::read_dir(&natives_path)?.count());
            if fs::read_dir(&natives_path)?.count() > 0 {
                return Ok(());
            }
        }

        for lib in &cache.package.libraries {
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
                R.set_message(t!("natives_download_missing").to_string());
                download_retry(&classifier_url, &native_file,&self.reporter).await?;
                extract_zip(&native_file, &natives_path)?;
            }
            R.add_progress(1.0);
        }

        Ok(())
    }
}
