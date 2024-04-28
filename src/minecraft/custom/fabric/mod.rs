use async_trait::async_trait;
use std::fs;

use crate::{
    error::FabricError,
    minecraft::{downloader::Downloader, version::MinecraftVersionBase},
    network::get_json,
    prelude::Result,
    reporter::Reporter,
};

use self::serde::{LoaderList, Package, VersionList};

pub mod serde;
#[derive(Clone)]

pub struct FabricVersion {
    pub loader_version: String,
    pub version: MinecraftVersionBase,
}

const META: &str = "https://meta.fabricmc.net/v2/";

pub async fn get_available_loaders() -> Result<LoaderList> {
    get_json::<LoaderList>(format!("{}{}", META, "versions/loader")).await
}

pub async fn get_available_versions() -> Result<VersionList> {
    get_json::<VersionList>(format!("{}{}", META, "versions/game")).await
}

pub async fn get_package_by_version(version: &String, loader_version: String) -> Result<Package> {
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
            Ok(get_json::<Package>(format!(
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
