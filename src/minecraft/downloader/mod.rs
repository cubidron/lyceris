use std::{
    fs::{self, read_dir},
    path::{PathBuf, MAIN_SEPARATOR_STR},
};

use async_trait::async_trait;
use futures_util::{future::try_join_all, lock::MutexGuard};
use tokio::try_join;

use crate::{
    network::download_retry,
    prelude::Result,
    reporter::{Case, Progress, Reporter},
    utils::{extract_zip, hash_file, hash_files},
};

use super::{
    java::get_manifest_by_version,
    version::{Custom, MinecraftVersion},
    Instance, Store, STORE,
};

pub struct File {
    pub hash: String,
    pub url: String,
    pub state: bool,
    pub path: PathBuf,
}

#[async_trait]
pub trait Downloader {
    async fn download_assets(&self, store: &MutexGuard<'_, Store>) -> Result<()>;
    async fn download_client(&self, store: &MutexGuard<'_, Store>) -> Result<()>;
    async fn download_libraries(&self, store: &MutexGuard<'_, Store>) -> Result<()>;
    async fn download_natives(&self, store: &MutexGuard<'_, Store>) -> Result<()>;
    async fn download_java(&self) -> Result<()>;
    async fn install(&self, store: &MutexGuard<'_, Store>) -> Result<()> {
        self.download_client(store).await?;
        self.download_assets(store).await?;
        self.download_libraries(store).await?;
        self.download_natives(store).await?;
        self.download_java().await?;
        Ok(())
    }
}
#[async_trait]
impl<R: Reporter> Downloader for Instance<R> {
    async fn download_assets(&self, store: &MutexGuard<'_, Store>) -> Result<()> {
        self.reporter
            .send(Case::SetMessage(t!("check_resources").to_string()));

        let index = &store.index.clone();

        let mut files: Vec<File> = vec![];

        self.reporter.send(Case::SetProgress(0.0));
        self.reporter
            .send(Case::SetMaxProgress(store.index.objects.len() as f64));

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
                self.reporter.send(Case::SetMessage(
                    t!("download_missing_resources").to_string(),
                ));
                download_retry(
                    format!(
                        "https://resources.download.minecraft.net/{sub_hash}/{}",
                        object.hash
                    ),
                    &hash_path,
                    &self.reporter,
                )
                .await?;
                self.reporter.send(Case::AddProgress(1.0));
            }
        }
        let files = hash_files(files)?;

        for file in files {
            if !file.state {
                download_retry(&file.url, &file.path, &self.reporter).await?;
            }
            self.reporter.send(Case::AddProgress(1.0));
        }

        Ok(())
    }

    async fn download_client(&self, store: &MutexGuard<'_, Store>) -> Result<()> {
        self.reporter
            .send(Case::SetMessage(t!("check_client").to_string()));
        self.reporter.send(Case::SetProgress(0.0));
        self.reporter.send(Case::SetMaxProgress(1.0));
        let file_path = self
            .config
            .root_path
            .join("versions")
            .join(&self.config.version_name)
            .join(format!("{}.jar", self.config.version_name));

        if file_path.is_file() && hash_file(&file_path)? == store.package.downloads.client.sha1 {
            self.reporter.send(Case::AddProgress(1.0));
            return Ok(());
        }

        self.reporter
            .send(Case::SetMessage(t!("install_client").to_string()));
        download_retry(
            store.package.downloads.client.url.clone(),
            &file_path,
            &self.reporter,
        )
        .await?;

        if let MinecraftVersion::Custom(_) = self.config.version {
            // todo: Implement a different logic in here
        } else {
            fs::write(
                self.config
                    .root_path
                    .join("versions")
                    .join(&self.config.version_name)
                    .join(format!("{}.json", self.config.version_name)),
                serde_json::to_string_pretty(&store.package).unwrap(),
            )?
        }
        self.reporter.send(Case::AddProgress(1.0));

        Ok(())
    }

    async fn download_libraries(&self, store: &MutexGuard<'_, Store>) -> Result<()> {
        self.reporter
            .send(Case::SetMessage(t!("check_libraries").to_string()));
        self.reporter.send(Case::SetProgress(0.0));
        self.reporter
            .send(Case::SetMaxProgress(store.package.libraries.len() as f64));
        for lib in &store.package.libraries {
            if let Some(artifact) = &lib.downloads.artifact {
                let file_path = self
                    .config
                    .root_path
                    .join("libraries")
                    .join(artifact.path.replace('/', MAIN_SEPARATOR_STR));

                if !self.parse_rule(lib)
                    && (!file_path.is_file() || hash_file(&file_path)? != artifact.sha1)
                {
                    self.reporter.send(Case::SetMessage(
                        t!("download_missing_libraries").to_string(),
                    ));
                    download_retry(&artifact.url, &file_path, &self.reporter).await?;
                }
            }
            self.reporter.send(Case::AddProgress(1.0));
        }

        if let super::version::MinecraftVersion::Custom(ext) = &self.config.version {
            match ext {
                Custom::Fabric(v) => {
                    if let Some(package) = &v.package {
                        self.reporter
                            .send(Case::SetMessage(t!("check_fabric").to_string()));
                        self.reporter
                            .send(Case::SetMaxProgress(package.libraries.len() as f64));
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
                                self.reporter.send(Case::SetMessage(
                                    t!("download_missing_fabric").to_string(),
                                ));
                                download_retry(&url, &path, &self.reporter).await?;
                            } else if let Some(sha1) = &i.sha1 {
                                if &hash_file(&path)? != sha1 {
                                    download_retry(&url, &path, &self.reporter).await?;
                                }
                            }
                            self.reporter.send(Case::AddProgress(1.0));
                        }
                    }
                }
                Custom::Quilt(v) => {
                    if let Some(package) = &v.package {
                        self.reporter
                            .send(Case::SetMessage(t!("check_quilt").to_string()));
                        self.reporter
                            .send(Case::SetMaxProgress(package.libraries.len() as f64));
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
                                self.reporter.send(Case::SetMessage(
                                    t!("download_missing_quilt").to_string(),
                                ));
                                download_retry(&url, &path, &self.reporter).await?;
                            } else if let Some(sha1) = &i.sha1 {
                                if &hash_file(&path)? != sha1 {
                                    download_retry(&url, &path, &self.reporter).await?;
                                }
                            }
                            self.reporter.send(Case::AddProgress(1.0));
                        }
                    }
                },
                Custom::OptiFine(v)=> {
                    &v.generate_files(&self.config.root_path)?;
                }
                _ => unimplemented!(),
            }
        }
        Ok(())
    }

    async fn download_java(&self) -> Result<()> {
        self.reporter
            .send(Case::SetMessage(t!("check_java").to_string()));
        self.reporter.send(Case::SetProgress(0.0));
        let manifest = get_manifest_by_version(&self.config.java_version).await?;
        let java_path = self
            .config
            .java_path
            .join(self.config.java_version.to_string());
        self.reporter
            .send(Case::SetMaxProgress(manifest.files.len() as f64));
        for (name, file) in manifest.files {
            let path = java_path.join(name);
            if let Some(downloads) = file.downloads {
                if path.is_file() && hash_file(&path)? == downloads.raw.sha1 {
                    self.reporter.send(Case::AddProgress(1.0));
                    continue;
                }
                self.reporter.send(Case::SetMessage(
                    t!("download_missing_java").to_string(),
                ));
                download_retry(&downloads.raw.url, &path, &self.reporter).await?;
            }
            self.reporter.send(Case::AddProgress(1.0));
        }

        Ok(())
    }

    async fn download_natives(&self, store: &MutexGuard<'_, Store>) -> Result<()> {
        let mut classifier_url = String::new();
        self.reporter
            .send(Case::SetMessage(t!("check_natives").to_string()));
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

        for lib in &store.package.libraries {
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
                self.reporter.send(Case::SetMessage(
                    t!("download_missing_natives").to_string(),
                ));
                download_retry(&classifier_url, &native_file, &self.reporter).await?;
                extract_zip(&native_file, &natives_path)?;
            }
        }

        Ok(())
    }
}
