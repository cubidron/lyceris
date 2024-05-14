use async_trait::async_trait;
use std::fs;

use crate::{
    error::{FabricError, QuiltError},
    minecraft::{downloader::Downloader, serde::Package, version::MinecraftVersionBase},
    network::get_json,
    prelude::Result,
    reporter::Reporter,
};

use self::serde::{LoaderList, Package as QuiltPackage, VersionList};

pub mod serde;
#[derive(Clone,Default)]

pub struct Quilt {
    pub version: MinecraftVersionBase,
    pub loader_version: String,
    pub package : Option<QuiltPackage>
}

impl Quilt{
    pub fn new(version : MinecraftVersionBase, loader_version: String) -> Self{
        Self { version,loader_version, package: None }
    }
}

const META: &str = "https://meta.quiltmc.org/v3/";

pub async fn get_available_loaders() -> Result<LoaderList> {
    get_json::<LoaderList>(format!("{}{}", META, "versions/loader")).await
}

pub async fn get_available_versions() -> Result<VersionList> {
    get_json::<VersionList>(format!("{}{}", META, "versions/game")).await
}

pub async fn get_package_by_version(version: String, loader_version: String) -> Result<QuiltPackage> {
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
            Ok(get_json::<QuiltPackage>(format!(
                "{}versions/loader/{}/{}/profile/json",
                META, version, loader_version
            ))
            .await?)
        } else {
            Err(crate::error::Error::QuiltError(
                QuiltError::PackageNotFound,
            ))
        }
    } else {
        Err(crate::error::Error::QuiltError(
            QuiltError::PackageNotFound,
        ))
    }
}
