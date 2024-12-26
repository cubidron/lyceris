use super::Loader;
use crate::{
    error::Error,
    http::fetch::fetch,
    json::version::meta::{
        custom::CustomMeta,
        vanilla::{self, VersionMeta},
    },
    minecraft::{config::Config, emitter::Emitter, parse::parse_lib_path},
};
use serde::{Deserialize, Serialize};

const VERSION_META_ENDPOINT: &str = "https://meta.fabricmc.net/v2/";

#[derive(Serialize, Deserialize)]
struct FabricLoader {
    separator: Separator,
    build: i64,
    maven: String,
    version: String,
    stable: bool,
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
    version: String,
    stable: bool,
}

pub struct Fabric(pub &'static str);

impl Loader for Fabric {
    async fn merge<T: Loader>(
        &self,
        _config: &Config<T>,
        mut meta: VersionMeta,
        _emitter: Option<&Emitter>,
    ) -> crate::Result<VersionMeta> {
        let loaders: Vec<FabricLoader> =
            fetch(format!("{}versions/loader", VERSION_META_ENDPOINT)).await?;
        let versions: Vec<Version> =
            fetch(format!("{}versions/game", VERSION_META_ENDPOINT)).await?;

        let loader = loaders
            .into_iter()
            .find(|v| v.version == self.0)
            .ok_or_else(|| Error::UnknownVersion("Fabric Loader".into()))?;
        let fabric = versions
            .into_iter()
            .find(|v| v.version == meta.id)
            .ok_or_else(|| Error::UnknownVersion("Fabric".into()))?;

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
                .filter_map(|lib| {
                    let path = parse_lib_path(&lib.name).ok()?;
                    lib.url.map(|url| vanilla::Library {
                        downloads: Some(vanilla::LibraryDownloads {
                            artifact: Some(vanilla::File {
                                path: Some(path.clone()),
                                sha1: lib.sha1.unwrap_or_default(),
                                size: lib.size.unwrap_or_default(),
                                url: format!("{}/{}", url, path),
                            }),
                            classifiers: None,
                        }),
                        extract: None,
                        name: lib.name.clone(),
                        rules: None,
                        natives: None,
                        skip_args: false,
                    })
                })
                .collect::<Vec<_>>(),
        );

        if let Some(ref mut arguments) = meta.arguments {
            if let Some(jvm) = version.arguments.jvm {
                arguments.jvm.extend(jvm);
            }
            if let Some(game) = version.arguments.game {
                arguments.game.extend(game);
            }
        }

        meta.main_class = version.main_class;

        Ok(meta)
    }

    fn get_version(&self) -> String {
        self.0.to_string()
    }
}