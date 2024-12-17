use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    http::fetch::fetch,
    json::version::meta::{
        custom::CustomMeta,
        vanilla::{self, VersionMeta},
    },
};

use super::Loader;

const VERSION_META_ENDPOINT: &str = "https://meta.quiltmc.org/v3/";

#[derive(Serialize, Deserialize)]
struct QuiltLoader {
    #[serde(rename = "separator")]
    pub separator: Separator,

    #[serde(rename = "build")]
    pub build: i64,

    #[serde(rename = "maven")]
    pub maven: String,

    #[serde(rename = "version")]
    pub version: String,
}

#[derive(Serialize, Deserialize)]
enum Separator {
    #[serde(rename = "+build.")]
    Build,

    #[serde(rename = ".")]
    Empty,
}

#[derive(Serialize, Deserialize)]
struct Version {
    #[serde(rename = "version")]
    pub version: String,

    #[serde(rename = "stable")]
    pub stable: bool,
}

pub struct Quilt(pub String);

impl Loader for Quilt {
    async fn merge(&self, game_dir: &Path, mut meta: VersionMeta) -> crate::Result<VersionMeta> {
        let loaders: Vec<QuiltLoader> =
            fetch(format!("{}{}", VERSION_META_ENDPOINT, "versions/loader")).await?;
        let versions: Vec<Version> =
            fetch(format!("{}{}", VERSION_META_ENDPOINT, "versions/game")).await?;

        let loader = loaders
            .into_iter()
            .find(|v| v.version == self.0)
            .ok_or(Error::UnknownVersion("Quilt Loader".into()))?;
        let fabric = versions
            .into_iter()
            .find(|v| v.version == meta.id)
            .ok_or(Error::UnknownVersion("Quilt".into()))?;

        let version: CustomMeta = fetch(format!(
            "{}versions/loader/{}/{}/profile/json",
            VERSION_META_ENDPOINT, fabric.version, loader.version
        ))
        .await?;

        meta.libraries.retain(|lib| {
            version
                .libraries
                .iter()
                .all(|v_lib| v_lib.name.split(':').nth(1) != lib.name.split(':').nth(1))
        });

        meta.libraries.extend(
            version
                .libraries
                .into_iter()
                .map(|lib| {
                    let parts = lib.name.split(':').collect::<Vec<_>>();
                    let file_name = format!("{}-{}.jar", parts[1], parts[2]);
                    let path = game_dir
                        .join("libraries")
                        .join(parts[0].replace('.', std::path::MAIN_SEPARATOR_STR))
                        .join(parts[1])
                        .join(parts[2])
                        .join(&file_name);
                    let url = format!(
                        "{}{}/{}/{}/{}",
                        lib.url,
                        parts[0].replace('.', "/"),
                        parts[1],
                        parts[2],
                        file_name
                    );

                    vanilla::Library {
                        downloads: Some(vanilla::LibraryDownloads {
                            artifact: Some(vanilla::File {
                                path: Some(path.to_string_lossy().into_owned()),
                                sha1: lib.sha1.unwrap_or_default(),
                                size: lib.size.unwrap_or_default(),
                                url,
                            }),
                            classifiers: None,
                        }),
                        extract: None,
                        name: lib.name.clone(),
                        rules: None,
                        natives: None,
                    }
                })
                .collect::<Vec<_>>(),
        );

        if let Some(ref mut arguments) = meta.arguments {
            if let Some(jvm) = version.arguments.jvm {
                arguments.jvm.extend(jvm);
            }
            arguments.game.extend(version.arguments.game);
        }

        meta.main_class = version.main_class;

        Ok(meta)
    }
}
