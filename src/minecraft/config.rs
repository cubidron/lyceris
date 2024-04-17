use crate::{prelude::Result, reporter::Reporter};

use super::{
    custom::CustomPackage, downloader::ServerFile, java::get_manifest_by_version, serde::{Index, Package, VersionManifest}
};

#[derive(Default)]
pub struct Config {
    pub version_manifest: VersionManifest,
    pub package: Package,
    pub classpaths: Option<String>,
    pub custom: Option<CustomPackage>,
    pub index: Option<Index>,
    pub server_files : Option<Vec<ServerFile>>
}



impl Config {
    pub fn new(
        version_manifest: VersionManifest,
        package: Package,
        classpaths: Option<String>,
        custom: Option<CustomPackage>,
        index: Option<Index>,
        server_files : Option<Vec<ServerFile>>
    ) -> Self {
        Self {
            version_manifest,
            package,
            classpaths,
            custom,
            index,
            server_files
        }
    }

    pub async fn get_global_progress(&self) -> Result<f64> {
        let mut progress = 0.0;

        progress += self.package.libraries.len() as f64 * 3.0;
        progress += 1.0;
        if let Some(package) = &self.custom {
            match package {
                CustomPackage::Fabric(package) => {
                    progress += package.libraries.len() as f64 * 2.0;
                }
            }
        }
        if let Some(index) = &self.index{
            progress += index.objects.len() as f64;
        }
        if let Some(files) = &self.server_files{
            progress += files.len() as f64;
        }
        if let Some(version) = &self.package.java_version{
            let manifest = get_manifest_by_version(&version.convert()).await?;
            progress += manifest.files.len() as f64;
        }
        Ok(progress)
    }
}
