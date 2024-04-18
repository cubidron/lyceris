use directories::BaseDirs;
use once_cell::sync::Lazy;
use std::{collections::HashSet, process::Stdio};
use tokio::{fs::File, io::AsyncWriteExt,process::{Child, Command}};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;

use crate::{
    error::Error,
    minecraft::{auth::AuthMethod, custom::fabric::get_package_by_version, version::ToString},
    network::Network,
    prelude::{Result, CLASSPATH_SEPERATOR},
    reporter::{Case, Reporter},
    utils::{json_from_file, recurse_files},
};

use core::fmt;
use std::{
    fmt::Debug,
    fs,
    path::{PathBuf, MAIN_SEPARATOR_STR},
};

use self::{
    auth::Offline,
    config::Config,
    custom::CustomPackage,
    downloader::{Downloader, ServerFile},
    java::JavaVersion,
    serde::{Action, GameElement, Index, Library, Name, VersionManifest},
    version::{MinecraftVersion, VERSION_MANIFEST_URL},
};

pub mod auth;
pub mod config;
pub mod custom;
pub mod downloader;
pub mod java;
pub mod serde;
pub mod version;

#[derive(Clone)]

pub struct ProgressionReporter {}

impl Reporter for ProgressionReporter {
    fn send(&self, case: crate::reporter::Case) {}
}

pub const LAUNCHER_API: &str = "https://launcher.baso.network";

pub static NETWORK: Lazy<Network<ProgressionReporter>> =
    Lazy::new(|| Network::default().with_reporter(ProgressionReporter {}));

static DIRECTORY_STRUCTURE: Lazy<Vec<&str>> = Lazy::new(|| {
    Vec::from([
        "assets",
        "assets/indexes",
        "assets/objects",
        "libraries",
        "natives",
        "versions",
        "mods",
        "saves",
        "screenshots",
        "shaderpacks",
        "resourcepacks",
    ])
});

static WHITELIST : Lazy<Vec<&str>> = Lazy::new(||{
    Vec::from([
        "assets",
        "libraries",
        "natives",
        "versions",
        "data",
        "saves",
        "screenshots",
        "logs",
        ".fabric",
        "launcher_settings.json",
        "iris.properties",
        "options.txt",
        "irisUpdateInfo.json",
        "resourcepacks",
        "runtime",
        "shaderpacks",
        "launcher-logs",
        ".replay-cache",
        "replay_recordings",
        "server-resource-packs",
    ])
});
pub struct Launcher<R: Reporter> {
    // Authentication method.
    pub authentication: AuthMethod,
    // Root directory of Minecraft files. Example : %APPDATA%/.minecraft
    pub root_path: PathBuf,
    // Minecraft version.
    pub version: MinecraftVersion,
    // Version name in case that you're using custom version names. It only changes name of the file in versions folder.
    pub version_name: String,
    // Allocated memory. Example Memory::Gigabyte(2,2), Memory::Megabyte(2048,2048)
    pub memory: Memory,
    // Java path. it might be an executable or runtimes folder that structured from JAVA_RUNTIME.
    pub java_path: PathBuf,
    // Java version. You can specify it directly or let library handle it.
    pub java_version: JavaVersion,
    // Custom java arguments.
    pub custom_java_args: Vec<String>,
    // Custom launch arguments.
    pub custom_launch_args: Vec<String>,
    // Config for storing config parameters.
    pub config: Config,
    // Reporter for progression.
    pub reporter: Option<R>,
}

impl<R: Reporter> Launcher<R> {
    // A new function to generate the structer.
    // It will use 2 GB maximum and minimum memory and Cardinal as username.
    // ! IMPROVE INITIALIZATION.
    pub fn new(root_path: PathBuf, version: MinecraftVersion) -> Self {
        // Todo OS Support
        // Tries to get java path by the Java CLI, if can't it will use {root_path}/runtime/{java_version}/bin/java.exe
        let java_path: PathBuf = root_path
            .join("runtime");

        if root_path.is_dir() {
            fs::create_dir_all(&root_path);
        }

        if java_path.parent().unwrap().is_dir() {
            fs::create_dir_all(java_path.parent().unwrap());
        }

        for directory in DIRECTORY_STRUCTURE.iter() {
            let path = root_path.join(directory);
            if !path.is_dir() {
                fs::create_dir_all(path).unwrap();
            }
        }

        Self {
            authentication: AuthMethod::Offline(Offline {
                username: "Cardinal".to_string(),
            }),
            root_path,
            version: version.clone(),
            version_name: version.to_string(),
            memory: Memory::Gigabyte(2, 2),
            java_path,
            java_version: version.get_compatible_java_version(),
            custom_java_args: vec![],
            custom_launch_args: vec![],
            config: Config::default(),
            reporter: None,
        }
    }
    pub async fn initialize_config(mut self) -> Result<Self> {
        self.reporter.send(Case::SetMessage(
            "Minecraft sürüm bilgileri yükleniyor".to_string()
        ));

        // If package id is default that means config has not been initialized yet.
        if self.config.package.id == String::default() {
            let version_manifest_path: PathBuf = self
                .root_path
                .join("assets")
                .join("indexes")
                .join("version_manifest.json");

            // If version manifest file is exists, we use it.
            let version_manifest = if version_manifest_path.is_file() {
                json_from_file(version_manifest_path)?
            } else {
                let manifest = NETWORK
                    .get_json::<VersionManifest>(VERSION_MANIFEST_URL)
                    .await?;
                let mut file = File::create(&version_manifest_path).await?;
                self.reporter.send(Case::SetSubMessage(format!(
                    "Saving manifest file at {}",
                    version_manifest_path.display()
                )));
                file.write_all(serde_json::to_string_pretty(&manifest)?.as_bytes())
                    .await?;
                manifest
            };
            // Taking the package by filtering version_manifest.versions
            let package = NETWORK
                .get_json(
                    &version_manifest
                        .clone()
                        .versions
                        .into_iter()
                        .filter(|x| x.version_type == self.version.get_type())
                        .find(|x| x.id == self.version.to_string())
                        .unwrap()
                        .url,
                )
                .await?;
            match self.version.clone() {
                MinecraftVersion::Custom(ext) => match ext {
                    version::Custom::Fabric(version) => {
                        let version_path = self.root_path.join("versions").join(&self.version_name);
                        let fabric_package = if version_path
                            .join(format!("{}.json", &self.version_name))
                            .exists()
                        {
                            if let Ok(data) = fs::read_to_string(
                                version_path.join(format!("{}.json", &self.version_name)),
                            ) {
                                serde_json::from_str(data.as_str())?
                            } else {
                                get_package_by_version(
                                    &version.version.to_string(),
                                    version.loader_version,
                                )
                                .await?
                            }
                        } else {
                            get_package_by_version(
                                &version.version.to_string(),
                                version.loader_version,
                            )
                            .await?
                        };

                        if !version_path.is_dir() {
                            fs::create_dir_all(&version_path).ok();
                        }

                        fs::write(
                            version_path.join(format!("{}.json", &self.version_name)),
                            serde_json::to_string_pretty::<custom::fabric::serde::Package>(
                                &fabric_package,
                            )?,
                        )
                        .ok();
                        self.config = Config::new(
                            version_manifest,
                            package,
                            None,
                            Some(CustomPackage::Fabric(fabric_package)),
                            None,
                            None,
                        );
                    }
                    _ => {
                        unimplemented!()
                    }
                },
                _ => {
                    self.config = Config::new(version_manifest, package, None, None, None,None);
                }
            }
        }
        let index_path = self.root_path.join("assets").join("indexes").join(format!(
            "{}.json",
            self.config.package.asset_index.id.clone()
        ));

        let index: Index = if index_path.is_file() {
            json_from_file::<Index>(index_path)?
        } else {
            NETWORK.download_with_retry(&self.config.package.asset_index.url, &index_path)
                .await?;
            json_from_file::<Index>(index_path)?
        };
        let server_files = NETWORK
            .get_json::<Vec<ServerFile>>(format!("{}/api/files", LAUNCHER_API))
            .await?;
        self.config.server_files = Some(server_files);
        
        self.config.index = Some(index);

        Ok(self)
    }

    pub async fn launch(mut self) -> Result<Child> {
        self = self.initialize_config().await?;

        self.reporter
            .send(Case::SetMaxProgress(self.config.get_global_progress().await?));

        let mut downloader = Downloader::new(&self);
        downloader.download_assets().await?;

        downloader.download_libraries().await?;

        downloader.download_client().await?;

        downloader.download_natives().await?;
        
        // downloader.download_custom().await?;

        // self.validate()?;

        self.java_path = downloader.download_java().await?;

        let (mut game, mut jvm) = (Vec::<String>::new(), Vec::<String>::new());

        match self.memory {
            Memory::Gigabyte(min, max) => {
                jvm.push(format!("-Xms{}G", min));
                jvm.push(format!("-Xmx{}G", max));
            }
            Memory::Megabyte(min, max) => {
                jvm.push(format!("-Xms{}M", min));
                jvm.push(format!("-Xmx{}M", max));
            }
        }

        let classpaths = self.get_classpaths().await?;

        self.reporter
            .send(Case::SetMessage("Argümanlar ayarlanıyor".to_string()));

        match self.config.package.arguments {
            Some(arguments) => {
                let username = match &self.authentication {
                    AuthMethod::Offline(offline_user) => {
                        offline_user.username.to_string()
                    }
                    _ => unimplemented!(), //AuthMethod::Online(microsoft_user) => microsoft_user.username,
                };
                for argument in arguments.game {
                    if let GameElement::String(string) = argument{
                        game.push(match string.as_str() {
                            // todo authentication
                            "${auth_player_name}" => username.clone(),
                            "${version_name}" => self.version_name.clone(),
                            "${game_directory}" => self.root_path.display().to_string(),
                            "${assets_root}" => {
                                self.root_path.join("assets").display().to_string()
                            }
                            "${assets_index_name}" => {
                                self.config.package.asset_index.id.clone()
                            }
                            "${auth_uuid}" => {
                                "bc58f189-ef1a-4bca-9e4f-e047ee4432be".to_string()
                            }
                            "${auth_access_token}" => "123".to_string(),
                            "${clientid}" => "123".to_string(),
                            "${auth_xuid}" => "123".to_string(),
                            "${user_type}" => "mojang".to_string(),
                            "${version_type}" => "release".to_string(),
                            _ => string.to_string(),
                        });
                    }
                }
                for argument in arguments.jvm {
                    if let serde::JvmElement::String(mut string) = argument{
                        if string.contains("${natives_directory}") {
                            string = string.replace(
                                "${natives_directory}",
                                &self
                                    .root_path
                                    .join("natives")
                                    .join(self.version.to_string())
                                    .display()
                                    .to_string(),
                            );
                        } else if string.contains("${launcher_name}") {
                            string = string.replace("${launcher_name}", "Cardinal")
                        } else if string.contains("${launcher_version}") {
                            string =
                                string.replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
                        } else if string.contains("${classpath}") {
                            string = string.replace("${classpath}", classpaths.as_str());
                            string.push_str(
                                &self
                                    .root_path
                                    .join("versions")
                                    .join(&self.version_name)
                                    .join(format!("{}.jar", self.version_name))
                                    .display()
                                    .to_string(),
                            );
                        }
                        jvm.push(string);
                    }
                }
                if let Some(custom) = &self.config.custom {
                    match custom {
                        CustomPackage::Fabric(fabric) => {
                            jvm.push(fabric.main_class.clone());
                        }
                    }
                } else {
                    jvm.push(self.config.package.main_class.clone());
                }
            }
            None => match self.config.package.minecraft_arguments {
                Some(arguments) => {
                    let arguments: Vec<String> =
                        arguments.split(' ').map(|x| x.to_string()).collect();
                    let version_path = self
                        .root_path
                        .join("versions")
                        .join(&self.version_name)
                        .join(format!("{}.jar", self.version_name))
                        .display()
                        .to_string();
                    jvm.push(format!(
                        "-Djava.library.path={}",
                        self.root_path
                            .join("natives")
                            .join(self.version.to_string())
                            .display()
                    ));
                    jvm.push(format!("-Dminecraft.client.jar={}", version_path));
                    jvm.push("-cp".to_string());
                    jvm.push(format!("{}{}", classpaths, version_path));

                    jvm.push(self.config.package.main_class.clone());
                    for arg in arguments {
                        let username = match &self.authentication {
                            AuthMethod::Offline(offline_user) => offline_user.username.to_string(),
                            AuthMethod::Online(microsoft_user) => unimplemented!(),
                        };
                        game.push(match arg.as_str() {
                            // todo authentication
                            "${auth_player_name}" => username,
                            "${version_name}" => self.version_name.clone(),
                            "${game_directory}" => self.root_path.display().to_string(),
                            "${assets_root}" => self.root_path.join("assets").display().to_string(),
                            "${assets_index_name}" => self.config.package.asset_index.id.clone(),
                            "${auth_uuid}" => "123".to_string(),
                            "${auth_access_token}" => "123".to_string(),
                            "${clientid}" => "123".to_string(),
                            "${auth_xuid}" => "123".to_string(),
                            "${user_type}" => "mojang".to_string(),
                            "${version_type}" => "release".to_string(),
                            "${user_properties}" => "{}".to_string(),
                            "${game_assets}" => match &self.version {
                                MinecraftVersion::Release((_, v1, v2)) => {
                                    if v1 < &8 {
                                        self.root_path
                                            .join("assets")
                                            .join("virtual")
                                            .join("legacy")
                                            .display()
                                            .to_string()
                                    } else {
                                        self.root_path.join("assets").display().to_string()
                                    }
                                }
                                MinecraftVersion::OldAlpha(v) => {
                                    format!("legacy/{}", self.version)
                                }
                                MinecraftVersion::OldBeta(v) => {
                                    format!("legacy/{}", self.version)
                                }
                                MinecraftVersion::Snapshot(v) => {
                                    format!("legacy/{}", self.version)
                                }
                                MinecraftVersion::Custom(v) => {
                                    self.root_path.join("assets").display().to_string()
                                }
                            },
                            _ => arg.to_string(),
                        });
                    }
                }
                None => {
                    unimplemented!()
                }
            },
        }

        jvm.append(&mut game);
        self.reporter
            .send(Case::SetMessage("Oyun başlatılıyor".to_string()));

        // `creation_flags` method avoids console window.
        #[cfg(target_os = "windows")]{
            let child = Command::new(self.java_path.join("bin").join("java.exe"))
            .current_dir(self.root_path)
            .args(jvm)
            .stdout(Stdio::piped())
            .creation_flags(0x08000000)
            .spawn()
            .expect("Failed to launch game");

            self.reporter.send(Case::RemoveProgress);
            Ok(child)
        }
        

        #[cfg(any(target_os = "linux", target_os = "macos"))]{
            let path = self.java_path.join("bin").join("java");
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
    
    async fn get_classpaths(&self) -> Result<String> {
        if let Some(cp) = &self.config.classpaths {
            return Ok(cp.to_string());
        }

        let mut cp = String::new();

        self.reporter
            .send(Case::SetMessage("Sınıf yolları ayarlanıyor".to_string()));

        // Iterating through package libraries to find classpaths.
        for lib in &self.config.package.libraries {
            // If classpath is installable it must have artifact property.
            if let Some(artifact) = &lib.downloads.artifact {
                // Parsing the rule for operating system.
                if !self.parse_rule(lib) {
                    let cp_path = self
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
                        .root_path
                        .join("libraries")
                        .join(natives.path.replace('/', std::path::MAIN_SEPARATOR_STR));

                    cp.push_str(
                        format!(
                            "{}{}",
                            self.root_path
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

        if let Some(package) = &self.config.custom {
            match package {
                CustomPackage::Fabric(package) => {
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

    pub fn validate(&self) -> Result<()>{
        if let Some(files) = &self.config.server_files{
            let files = files.iter().map(|f| self.root_path.join(f.path.replace("\\root\\baso-api\\files\\",""))).collect::<HashSet<PathBuf>>();
            if let Ok(local) = recurse_files(&self.root_path){
                let difference : HashSet<&PathBuf> = local.difference(&files).collect();
                for file in difference{
                    if WHITELIST.iter().any(|w| {
                        if let Some(_) = PathBuf::from(w).extension(){
                            file.display().to_string().contains(w)
                        }
                        else {
                            file.display().to_string().contains(format!("\\{}\\",w).as_str())    
                        }
                    } ){
                        continue;
                    }
                    if let Some(ext) = file.extension(){
                        if ext.eq("deactive"){
                            continue;
                        }
                    }
                    fs::remove_file(file).ok();
                }
            }
        }
        Ok(())
    }

    fn parse_rule(&self, lib: &Library) -> bool {
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

    pub fn with_reporter(mut self, reporter: R) -> Self {
        self.reporter = Some(reporter);
        self
    }

    pub fn with_root_path(mut self, root_path: PathBuf) -> Self {
        self.root_path = root_path;
        self
    }

    pub fn with_version(mut self, version: MinecraftVersion) -> Self {
        self.version = version;
        self
    }

    pub fn with_java_path(mut self, java_path: PathBuf) -> Self {
        self.java_path = java_path;
        self
    }

    pub fn with_java_version(mut self, java_version: JavaVersion) -> Self {
        self.java_version = java_version;
        self
    }

    pub fn with_custom_java_args(mut self, custom_java_args: Vec<String>) -> Self {
        self.custom_java_args = custom_java_args;
        self
    }

    pub fn with_custom_launch_args(mut self, custom_launch_args: Vec<String>) -> Self {
        self.custom_launch_args = custom_launch_args;
        self
    }

    pub fn with_memory(mut self, memory: Memory) -> Self {
        self.memory = memory;
        self
    }

    pub fn with_version_name(mut self, version_name: String) -> Self {
        self.version_name = version_name;
        self
    }

    pub fn with_authentication(mut self, authentication: AuthMethod) -> Self {
        self.authentication = authentication;
        self
    }
}

pub enum Memory {
    Gigabyte(u16, u16),
    Megabyte(u64, u64),
}
