use ::serde::Deserialize;
use std::{fmt::format, fs, path::PathBuf};

use crate::{
    error::FabricError,
    minecraft::{downloader::Downloader, json::Package, version::MinecraftVersionBase},
    network::get_json,
    prelude::Result,
    reporter::Reporter,
    utils::{extract_file_from_jar, json_from_file, read_file_from_jar},
};

pub mod json;
use self::json::Package as OptiFinePackage;

#[derive(Clone, Default, Deserialize)]

pub struct OptiFine {
    pub version: MinecraftVersionBase,
    pub jar_path: PathBuf,
    pub json_path: PathBuf,
    pub package: Option<OptiFinePackage>,
}

impl OptiFine {
    pub fn new(
        version: MinecraftVersionBase,
        jar_path: PathBuf,
        json_path: PathBuf,
        package: Option<OptiFinePackage>,
    ) -> Self {
        Self {
            version,
            jar_path,
            json_path,
            package: None,
        }
    }

    pub fn generate_files(&self, root_path: &PathBuf) -> Result<()> {
        let package = if let Some(package) = &self.package {
            package
        } else {
            &json_from_file::<OptiFinePackage>(&self.json_path)?
        };
        let id_split = package.id.split("-").collect::<Vec<&str>>();
        let version = id_split[0];
        let optifine_version = id_split[1].replace("OptiFine", "");
        let optifine_file_path = &root_path
            .join("libraries")
            .join("optifine")
            .join("OptiFine")
            .join(format!("{}{}", &version, &optifine_version))
            .join(format!("OptiFine-{}{}.jar", &version, &optifine_version));
        let launcherwrapper_version = read_file_from_jar(&self.jar_path, "launchwrapper-of.txt")?;
        let file_name = format!("launchwrapper-of-{}.jar", &launcherwrapper_version);
        let extract_location = root_path
            .join("libraries")
            .join("optifine")
            .join("launchwrapper-of")
            .join(&launcherwrapper_version)
            .join(&file_name);
        if !extract_location.parent().unwrap().is_dir() || !extract_location.is_file() {
            fs::create_dir_all(extract_location.parent().unwrap())?;
            extract_file_from_jar(&self.jar_path, &file_name, &extract_location)?;
        }
        if !optifine_file_path.parent().unwrap().is_dir() || !optifine_file_path.is_file() {
            fs::create_dir_all(optifine_file_path.parent().unwrap())?;
            fs::copy(&self.jar_path, optifine_file_path)?;
        }

        Ok(())
    }
}
