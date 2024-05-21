use std::fs;

use ::serde::Deserialize;

use crate::{
    error::FabricError,
    minecraft::{downloader::Downloader, serde::Package, version::MinecraftVersionBase},
    network::get_json,
    prelude::Result,
    reporter::Reporter,
};

use self::serde::{LoaderList, Package as FabricPackage, VersionList};

pub mod serde;
#[derive(Clone,Default,Deserialize)]

pub struct Fabric {
    pub version: MinecraftVersionBase,
    pub loader_version: String,
    pub package : Option<FabricPackage>
}

impl Fabric{
    pub fn new(version : MinecraftVersionBase, loader_version: String) -> Self{
        Self { version,loader_version, package: None }
    }
}

const META: &str = "https://meta.fabricmc.net/v2/";

pub async fn get_available_loaders() -> Result<LoaderList> {
    get_json::<LoaderList>(format!("{}{}", META, "versions/loader")).await
}

pub async fn get_available_versions() -> Result<VersionList> {
    get_json::<VersionList>(format!("{}{}", META, "versions/game")).await
}

pub async fn get_package_by_version(version: String, loader_version: String) -> Result<FabricPackage> {
    if get_available_loaders()
        .await?
        .iter()
        .any(|l| l.version == loader_version)
    {
        if get_available_versions()
            .await?
            .iter()
            .any(|v| v.version == *version)
        {
            Ok(get_json::<FabricPackage>(format!(
                "{}versions/loader/{}/{}/profile/json",
                META, version, loader_version
            ))
            .await?)
        } else {
            Err(crate::error::Error::FabricError(
                FabricError::PackageNotFound,
            ))
        }
    } else {
        Err(crate::error::Error::FabricError(
            FabricError::PackageNotFound,
        ))
    }
}
