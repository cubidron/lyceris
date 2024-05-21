use crate::{
    error::Error,
    minecraft::{auth::AuthMethod, version::ToString},
    network::{download_retry, get, get_json},
    prelude::{Result, CLASSPATH_SEPERATOR, R},
    reporter::{Case, Reporter},
    utils::json_from_file,
};

use ::serde::{de::DeserializeOwned, Deserialize, Serialize};
use directories::BaseDirs;
use futures_util::lock::{Mutex, MutexGuard};
use lazy_static::lazy_static;
use log::{error, warn};
use once_cell::sync::Lazy;
use rust_i18n::t;
use std::{
    collections::HashSet,
    env,
    fmt::{self, Debug},
    fs,
    fs::File,
    io::Write,
    path::{PathBuf, MAIN_SEPARATOR_STR},
    process::Stdio,
};
use tokio::process::{Child, Command};

#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use self::{
    custom::fabric::{get_package_by_version, Fabric},
    downloader::Downloader,
    java::JavaVersion,
    serde::{Action, GameElement, Index, Library, Name, Package, VersionManifest},
    version::{Custom, MinecraftVersion, VERSION_MANIFEST_URL},
};

pub mod auth;
pub mod custom;
pub mod downloader;
pub mod java;
pub mod serde;
pub mod version;

lazy_static! {
    static ref CACHE: Mutex<Cache> = Mutex::new(Cache {
        version_manifest: VersionManifest::default(),
        package: Package::default(),
        index: Index::default(),
        classpaths: None,
    });
}
#[derive(Deserialize)]
pub enum Memory {
    Gigabyte(u16, u16),
    Megabyte(u64, u64),
}

#[derive(Deserialize)]
pub struct Config {
    // Authentication method. Default: Offline("Lyceris")
    pub authentication: AuthMethod,
    // Root directory of Minecraft files. Default: config_directory/.minecraft
    pub root_path: PathBuf,
    // Minecraft version. Default: 1.16
    pub version: MinecraftVersion,
    // Version name in case that you're using custom version names. It only changes name of the file in versions folder. Default: Same as version
    pub version_name: String,
    // Allocated memory. Example Memory::Gigabyte(2,2), Memory::Megabyte(2048,2048). Default: Memory::Gigabyte(2,2)
    pub memory: Memory,
    // Java path. it might be an executable or runtimes folder that structured from JAVA_RUNTIME.
    pub java_path: PathBuf,
    // Java version. You can specify it directly or let library handle it.
    pub java_version: JavaVersion,
    // Custom java arguments.
    pub custom_java_args: Vec<String>,
    // Custom launch arguments.
    pub custom_launch_args: Vec<String>,
}

pub struct Cache {
    version_manifest: VersionManifest,
    package: Package,
    index: Index,
    classpaths: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let default_version = MinecraftVersion::Release((1, 16, None));
        let root_path = if let Some(base_dirs) = directories::BaseDirs::new() {
            base_dirs.config_dir().join(".minecraft")
        } else {
            let current_path = PathBuf::from(".minecraft");
            current_path
        };
        
        Self {
            authentication: AuthMethod::Offline("Lyceris".to_string()),
            root_path: root_path.clone(),
            version: default_version.clone(),
            version_name: String::default(),
            memory: Memory::Gigabyte(2, 2),
            java_path: PathBuf::default(),
            java_version: default_version.get_compatible_java_version(),
            custom_java_args: vec![],
            custom_launch_args: vec![],
        }
    }
}

pub struct Instance<R: Reporter> {
    config: Config,
    reporter: Option<R>,
}

impl<R: Reporter> Instance<R> {
    pub fn new(config: Config, reporter: Option<R>) -> Self {
        Self { config, reporter }
    }
    pub async fn prepare<'a>(&mut self, cache: &'a mut MutexGuard<'_, Cache>) -> Result<()> {
        self.reporter
            .send(Case::SetMessage(t!("prepare").to_string()));

        if cache.package.id == String::default() {
            let version_manifest_path: PathBuf = self
                .config
                .root_path
                .join("assets")
                .join("indexes")
                .join("version_manifest.json");

            let version_manifest = if version_manifest_path.is_file() {
                json_from_file(version_manifest_path)?
            } else {
                let manifest = get_json::<VersionManifest>(VERSION_MANIFEST_URL).await?;
                let mut file = File::create(&version_manifest_path)?;
                self.reporter.send(Case::SetSubMessage(
                    t!("manifest_file_save", path = version_manifest_path.display()).to_string(),
                ));
                file.write_all(serde_json::to_string_pretty(&manifest)?.as_bytes())?;
                manifest
            };

            if let MinecraftVersion::Custom(ext) = &mut self.config.version {
                match ext {
                    Custom::Fabric(v) => {
                        self.config.version =
                            MinecraftVersion::Custom(Custom::Fabric(custom::fabric::Fabric {
                                version: v.version,
                                loader_version: v.loader_version.to_string(),
                                package: Some(
                                    get_package_by_version(
                                        v.version.to_string(),
                                        v.loader_version.to_string(),
                                    )
                                    .await?,
                                ),
                            }));
                    }
                    Custom::Quilt(v) => {
                        self.config.version =
                            MinecraftVersion::Custom(Custom::Quilt(custom::quilt::Quilt {
                                version: v.version,
                                loader_version: v.loader_version.to_string(),
                                package: Some(
                                    crate::minecraft::custom::quilt::get_package_by_version(
                                        v.version.to_string(),
                                        v.loader_version.to_string(),
                                    )
                                    .await?,
                                ),
                            }));
                    }
                    _ => unimplemented!(),
                }
            }

            let package = get_json(
                &version_manifest
                    .clone()
                    .versions
                    .into_iter()
                    .find(|x| x.id == self.config.version.to_string())
                    .unwrap()
                    .url,
            )
            .await?;

            {
                cache.package = package;
                cache.version_manifest = version_manifest;
            }
        }

        let index_path = self
            .config
            .root_path
            .join("assets")
            .join("indexes")
            .join(format!("{}.json", cache.package.asset_index.id.clone()));

        let index: Index = if index_path.is_file() {
            json_from_file::<Index>(index_path)?
        } else {
            download_retry(&cache.package.asset_index.url, &index_path, &self.reporter).await?;
            json_from_file::<Index>(index_path)?
        };

        if self.config.version_name.is_empty() {
            self.config.version_name = self.config.version.to_string().clone();
        }

        if self.config.java_path == PathBuf::default() {
            self.config.java_path = self.config.root_path.join("runtimes");
        }

        cache.index = index;

        Ok(())
    }

    pub async fn launch(&mut self) -> Result<Child> {
        let mut cache = CACHE.lock().await;

        self.prepare(&mut cache).await?;

        self.install(&cache).await?;

        let args = self.prepare_arguments(&mut cache)?;
        // `creation_flags` method avoids console window.

        self.reporter
            .send(Case::SetMessage(t!("launching").to_string()));

        println!("{:?}", args);
        #[cfg(target_os = "windows")]
        {
            let child = Command::new(
                self.config
                    .java_path
                    .join(self.config.java_version.to_string())
                    .join("bin")
                    .join("javaw.exe"),
            )
            .current_dir(&self.config.root_path)
            .args(args)
            .stdout(Stdio::piped())
            .creation_flags(0x08000000)
            .spawn()
            .expect("Failed to launch game");

            self.reporter.send(Case::RemoveProgress);
            Ok(child)
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let path = self.java_path.join("bin").join("javaw");
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms)?;
            let child = Command::new(path)
                .current_dir(self.root_path)
                .args(jvm)
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to launch game");

            self.reporter.send(Case::RemoveProgress);
            Ok(child)
        }
    }

    pub fn parse_rule(&self, lib: &Library) -> bool {
        if let Some(rules) = &lib.rules {
            if rules.len() > 1 {
                if rules[0].action == Action::Allow
                    && rules[1].action == Action::Disallow
                    && rules[1]
                        .os
                        .as_ref()
                        .map_or(false, |os| os.name != Self::get_os())
                {
                    return Self::get_os() == Name::Osx;
                } else {
                    return true;
                }
            } else if rules[0].action == Action::Allow && rules[0].os.is_some() {
                return rules[0]
                    .os
                    .as_ref()
                    .map_or(false, |os| os.name != Self::get_os());
            }
        }
        false
    }

    fn handle_progress(self, total: f64, current: f64) {
        self.reporter.send(Case::SetMaxSubProgress(total));
        self.reporter.send(Case::SetSubProgress(current));
    }

    fn prepare_arguments(&self, cache: &mut MutexGuard<'_, Cache>) -> Result<Vec<String>> {
        let (mut game, mut jvm) = (Vec::<String>::new(), Vec::<String>::new());

        match self.config.memory {
            Memory::Gigabyte(min, max) => {
                jvm.push(format!("-Xms{}G", min));
                jvm.push(format!("-Xmx{}G", max));
            }
            Memory::Megabyte(min, max) => {
                jvm.push(format!("-Xms{}M", min));
                jvm.push(format!("-Xmx{}M", max));
            }
        }
        let classpaths = self.get_classpaths(cache)?;

        self.reporter
            .send(Case::SetMessage(t!("arguments_set").to_string()));

        match &cache.package.arguments {
            Some(arguments) => {
                let username = match &self.config.authentication {
                    AuthMethod::Offline(offline_user) => offline_user,
                    _ => unimplemented!(), //AuthMethod::Online(microsoft_user) => microsoft_user.username,
                };
                for argument in &arguments.game {
                    if let GameElement::String(string) = argument {
                        game.push(match string.as_str() {
                            // todo authentication
                            "${auth_player_name}" => username.clone(),
                            "${version_name}" => self.config.version_name.clone(),
                            "${game_directory}" => self.config.root_path.display().to_string(),
                            "${assets_root}" => {
                                self.config.root_path.join("assets").display().to_string()
                            }
                            "${assets_index_name}" => cache.package.asset_index.id.clone(),
                            "${auth_uuid}" => "bc58f189-ef1a-4bca-9e4f-e047ee4432be".to_string(),
                            "${auth_access_token}" => "123".to_string(),
                            "${clientid}" => "123".to_string(),
                            "${auth_xuid}" => "123".to_string(),
                            "${user_type}" => "mojang".to_string(),
                            "${version_type}" => "release".to_string(),
                            _ => string.to_string(),
                        });
                    }
                }
                for argument in &arguments.jvm {
                    if let serde::JvmElement::String(mut string) = argument.clone() {
                        if string.contains("${natives_directory}") {
                            string = string.replace(
                                "${natives_directory}",
                                &self
                                    .config
                                    .root_path
                                    .join("natives")
                                    .join(self.config.version.to_string())
                                    .display()
                                    .to_string(),
                            );
                        } else if string.contains("${launcher_name}") {
                            string = string.replace("${launcher_name}", "Cardinal");
                        } else if string.contains("${launcher_version}") {
                            string =
                                string.replace("${launcher_version}", env!("CARGO_PKG_VERSION"));
                        } else if string.contains("${classpath}") {
                            string = string.replace("${classpath}", classpaths.as_str());
                            string.push_str(
                                &self
                                    .config
                                    .root_path
                                    .join("versions")
                                    .join(&self.config.version_name)
                                    .join(format!("{}.jar", self.config.version_name))
                                    .display()
                                    .to_string(),
                            );
                        }
                        jvm.push(string);
                    }
                }
                if let MinecraftVersion::Custom(ext) = &self.config.version {
                    match ext {
                        Custom::Fabric(v) => {
                            if let Some(package) = &v.package {
                                jvm.push(package.main_class.clone());
                            }
                        }
                        Custom::Quilt(v) => {
                            if let Some(package) = &v.package {
                                jvm.push(package.main_class.clone());
                            }
                        }
                        _ => unimplemented!(),
                    }
                } else {
                    jvm.push(cache.package.main_class.clone());
                }
            }
            None => match &cache.package.minecraft_arguments {
                Some(arguments) => {
                    let arguments: Vec<String> =
                        arguments.split(' ').map(|x| x.to_string()).collect();
                    let version_path = self
                        .config
                        .root_path
                        .join("versions")
                        .join(&self.config.version_name)
                        .join(format!("{}.jar", self.config.version_name))
                        .display()
                        .to_string();
                    jvm.push(format!(
                        "-Djava.library.path={}",
                        self.config
                            .root_path
                            .join("natives")
                            .join(self.config.version.to_string())
                            .display()
                    ));
                    jvm.push(format!("-Dminecraft.client.jar={}", version_path));
                    jvm.push("-cp".to_string());
                    jvm.push(format!("{}{}", classpaths, version_path));

                    jvm.push(cache.package.main_class.clone());
                    for arg in arguments {
                        let username = match &self.config.authentication {
                            AuthMethod::Offline(offline_user) => offline_user.to_string(),
                            AuthMethod::Online(microsoft_user) => unimplemented!(),
                        };
                        game.push(match arg.as_str() {
                            // todo authentication
                            "${auth_player_name}" => username,
                            "${version_name}" => self.config.version_name.clone(),
                            "${game_directory}" => self.config.root_path.display().to_string(),
                            "${assets_root}" => {
                                self.config.root_path.join("assets").display().to_string()
                            }
                            "${assets_index_name}" => cache.package.asset_index.id.clone(),
                            "${auth_uuid}" => "123".to_string(),
                            "${auth_access_token}" => "123".to_string(),
                            "${clientid}" => "123".to_string(),
                            "${auth_xuid}" => "123".to_string(),
                            "${user_type}" => "mojang".to_string(),
                            "${version_type}" => "release".to_string(),
                            "${user_properties}" => "{}".to_string(),
                            "${game_assets}" => match &self.config.version {
                                MinecraftVersion::Release((_, v1, v2)) => {
                                    if v1 < &8 {
                                        self.config
                                            .root_path
                                            .join("assets")
                                            .join("virtual")
                                            .join("legacy")
                                            .display()
                                            .to_string()
                                    } else {
                                        self.config.root_path.join("assets").display().to_string()
                                    }
                                }
                                _ => self.config.root_path.join("assets").display().to_string(),
                            },
                            _ => arg.to_string(),
                        });
                    }
                }
                None => {
                    unimplemented!();
                }
            },
        }
        jvm.append(&mut game);

        Ok(jvm)
    }

    fn get_classpaths(&self, cache: &mut MutexGuard<Cache>) -> Result<String> {
        if let Some(cp) = &cache.classpaths {
            return Ok(cp.to_string());
        }

        let mut cp = String::new();

        self.reporter
            .send(Case::SetMessage(t!("classpaths_set").to_string()));

        // Iterating through package libraries to find classpaths.
        for lib in &cache.package.libraries {
            // If classpath is installable it must have artifact property.
            if let Some(artifact) = &lib.downloads.artifact {
                // Parsing the rule for operating system.
                if !self.parse_rule(lib) {
                    let cp_path = self
                        .config
                        .root_path
                        .join("libraries")
                        .join(artifact.path.replace('/', MAIN_SEPARATOR_STR));
                    cp.push_str(format!("{}{}", cp_path.display(), CLASSPATH_SEPERATOR).as_str());
                }
            }
            // Find mappings for natives.
            let mut mapping = &None;
            if let Some(classifiers) = &lib.downloads.classifiers {
                if cfg!(target_os = "windows") {
                    mapping = &classifiers.natives_windows_64;
                } else if cfg!(target_os = "linux") {
                    mapping = &classifiers.natives_linux;
                } else if cfg!(target_os = "macos") {
                    mapping = &classifiers.natives_macos;
                } else {
                    panic!("Unsupported OS");
                }
                if let Some(natives) = mapping {
                    let classifier_path = self
                        .config
                        .root_path
                        .join("libraries")
                        .join(natives.path.replace('/', std::path::MAIN_SEPARATOR_STR));

                    cp.push_str(
                        format!(
                            "{}{}",
                            self.config
                                .root_path
                                .join("libraries")
                                .join(classifier_path)
                                .display(),
                            CLASSPATH_SEPERATOR
                        )
                        .as_str(),
                    );
                }
            }
            self.reporter.send(Case::AddProgress(1.0));
        }

        if let MinecraftVersion::Custom(ext) = &self.config.version {
            match ext {
                Custom::Fabric(v) => {
                    if let Some(package) = &v.package {
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
                            cp.push_str(format!("{};", path.display()).as_str());
                            self.reporter.send(Case::AddProgress(1.0));
                        }
                    }
                }
                Custom::Quilt(v) => {
                    if let Some(package) = &v.package {
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
                            cp.push_str(format!("{};", path.display()).as_str());
                            self.reporter.send(Case::AddProgress(1.0));
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(cp)
    }

    fn get_os() -> Name {
        if cfg!(target_os = "windows") {
            Name::Windows
        } else if cfg!(target_os = "linux") {
            Name::Linux
        } else if cfg!(target_os = "macos") {
            Name::Osx
        } else {
            panic!("Unsupported OS");
        }
    }
}
