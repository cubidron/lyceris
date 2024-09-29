mod json;

use std::{fs, path::PathBuf};

use ::serde::Deserialize;

use crate::{
    error::FabricError,
    minecraft::{downloader::Downloader, json::Package, version::MinecraftVersionBase},
    network::get_json,
    prelude::Result,
    reporter::Reporter,
};

use self::json::{Package as OptiFinePackage};

pub mod json;
#[derive(Clone,Default,Deserialize)]

pub struct OptiFine {
    pub version: MinecraftVersionBase,
    pub jar_path: PathBuf,
    pub json_path: PathBuf,
    pub package: Option<OptiFinePackage>
}

impl OptiFine{
    pub fn new(version: MinecraftVersionBase, jar_path: PathBuf, json_path: PathBuf, package: Option<OptiFinePackage>) -> Self{
        Self { version, jar_path, json_path, package: None }
    }
}