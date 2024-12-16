#![allow(async_fn_in_trait)]

use crate::json::version::meta::vanilla::VersionMeta;
use std::path::Path;

use super::error::MinecraftError;

pub mod fabric;
pub mod quilt;

pub trait Loader {
    async fn merge(
        &self,
        game_dir: &Path,
        meta: VersionMeta,
    ) -> Result<VersionMeta, MinecraftError>;
}

impl Loader for () {
    async fn merge(&self, _: &Path, meta: VersionMeta) -> Result<VersionMeta, MinecraftError> {
        Ok(meta)
    }
}
