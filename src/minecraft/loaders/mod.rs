#![allow(async_fn_in_trait)]

use crate::json::version::meta::vanilla::VersionMeta;
use std::path::Path;

pub mod fabric;
pub mod quilt;

pub trait Loader {
    async fn merge(
        &self,
        game_dir: &Path,
        meta: VersionMeta,
    ) -> crate::Result<VersionMeta>;
}

impl Loader for () {
    async fn merge(&self, _: &Path, meta: VersionMeta) -> crate::Result<VersionMeta> {
        Ok(meta)
    }
}
