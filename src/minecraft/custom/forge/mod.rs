use std::collections::HashMap;
use std::env::{self, temp_dir};
use std::fs::create_dir_all;
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use directories::{BaseDirs, UserDirs};
use json::InstallerProfile;
use log::info;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::error::Error;
use crate::network::{download, get_json};
use crate::utils::{extract_file_from_jar, json_from_file};

use self::json::Package as ForgePackage;
use crate::minecraft::version::MinecraftVersionBase;
use crate::prelude::Result;

pub mod args;
pub mod json;

macro_rules! processor_rules {
    ($dest:expr; $($name:literal : client => $client:expr, server => $server:expr;)+) => {
        $(std::collections::HashMap::insert(
            $dest,
            String::from($name),
            crate::minecraft::custom::forge::json::SidedDataEntry {
                client: String::from($client),
                server: String::from($server),
            },
        );)+
    }
}

macro_rules! wrap_ref_builder {
    ($id:ident = $init:expr => $transform:block) => {{
        let mut it = $init;
        {
            let $id = &mut it;
            $transform;
        }
        it
    }};
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forge {
    pub version: MinecraftVersionBase,
    pub loader_version: String,
    pub package: Option<ForgePackage>,
}

impl Forge {
    pub fn new(version: MinecraftVersionBase, loader_version: String) -> Self {
        Self {
            version,
            loader_version,
            package: None,
        }
    }
}

pub async fn get_package_by_version(
    root_path: &PathBuf,
    version: String,
    loader_version: String,
) -> Result<ForgePackage> {
    let file_url = format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{0}/forge-{0}-installer.jar",
        format!("{}-{}", version, loader_version)
    );

    let temp_folder = env::temp_dir().join(format!("lyceris-{}-{}", version, loader_version));

    let installer_path = temp_folder.join("installer.jar");
    let install_profile_path = temp_folder.join("install_profile.json");
    let version_path = temp_folder.join("version.json");

    if !installer_path.is_file() {
        download(file_url, &installer_path, &None::<()>).await?;
    }

    if !install_profile_path.is_file() {
        extract_file_from_jar(
            &installer_path,
            "install_profile.json",
            &install_profile_path,
        )?;
    }

    if !version_path.is_file() {
        extract_file_from_jar(&installer_path, "version.json", &version_path)?;
    }

    let install_profile_json = json_from_file::<InstallerProfile>(install_profile_path)?;
    let mut version_json = json_from_file::<ForgePackage>(version_path)?;

    version_json
        .libraries
        .extend(install_profile_json.libraries.into_iter().map(|mut x| {
            x.exclude = true;

            x
        }));
    version_json.processors = Some(install_profile_json.processors);
    version_json.data = Some(install_profile_json.data);

    for lib in version_json.libraries.iter_mut() {
        if let Some(downloads) = &mut lib.downloads {
            if let Some(artifact) = &downloads.artifact {
                if artifact.url.is_empty() {
                    if !&root_path.join("libraries").join(&artifact.path).exists() {
                        create_dir_all(
                            &root_path
                                .join("libraries")
                                .join(&artifact.path)
                                .parent()
                                .unwrap(),
                        )
                        .ok();

                        extract_file_from_jar(
                            &installer_path,
                            &format!("maven/{}", &artifact.path),
                            &root_path.join("libraries").join(&artifact.path),
                        )?;
                    }
                }
            } else {
                lib.url = Some(format!(
                    "https://maven.creeperhost.net/{}",
                    get_path_from_artifact(lib.name.as_str())?
                ));
            }
        } else {
            lib.url = Some(format!(
                "https://maven.creeperhost.net/{}",
                get_path_from_artifact(lib.name.as_str())?
            ));
        }
    }

    let mut new_data = HashMap::new();
    if let Some(ref data) = version_json.data {
        for (key, entry) in data {
            async fn extract_data(
                installer_path: &PathBuf,
                root_path: &PathBuf,
                key: &str,
                value: &str,
                libs: &mut Vec<crate::minecraft::custom::forge::json::Library>,
                version: &String,
            ) -> Result<String> {
                let file_name = value.split('/').last().ok_or_else(|| {
                    crate::error::Error::UnknownError(format!(
                        "Unable reading filename for data key {key} at path {value}",
                    ))
                })?;

                let mut file = file_name.split('.');
                let file_name = file.next().ok_or_else(|| {
                    crate::error::Error::UnknownError(format!(
                        "Unable reading filename only for data key {key} at path {value}",
                    ))
                })?;
                let ext = file.next().ok_or_else(|| {
                    crate::error::Error::UnknownError(format!(
                        "Unable reading extension only for data key {key} at path {value}",
                    ))
                })?;

                let path = format!(
                    "com.cubidron.lyceris:{}-installer-extracts:{}:{}@{}",
                    "forge", version, file_name, ext
                );

                if !&root_path
                    .join("libraries")
                    .join(get_path_from_artifact(&path)?)
                    .exists()
                {
                    create_dir_all(
                        &root_path
                            .join("libraries")
                            .join(get_path_from_artifact(&path)?)
                            .parent()
                            .unwrap(),
                    )
                    .ok();

                    extract_file_from_jar(
                        installer_path,
                        &value[1..value.len()],
                        &root_path
                            .join("libraries")
                            .join(get_path_from_artifact(&path)?),
                    )?
                }

                libs.push(crate::minecraft::custom::forge::json::Library {
                    md5: None,
                    sha1: None,
                    sha256: None,
                    sha512: None,
                    size: None,
                    downloads: None,
                    name: path.clone(),
                    url: None,
                    exclude: true,
                });

                Ok(format!("[{path}]"))
            }

            let client = if entry.client.starts_with('/') {
                extract_data(
                    &installer_path,
                    &root_path,
                    &key,
                    &entry.client,
                    &mut version_json.libraries,
                    &version_json.id,
                )
                .await?
            } else {
                entry.client.clone()
            };

            let server = if entry.server.starts_with('/') {
                extract_data(
                    &installer_path,
                    &root_path,
                    &key,
                    &entry.server,
                    &mut version_json.libraries,
                    &version_json.id,
                )
                .await?
            } else {
                entry.server.clone()
            };

            new_data.insert(
                key.clone(),
                crate::minecraft::custom::forge::json::SidedDataEntry { client, server },
            );
        }
    }

    version_json.data = Some(new_data);
    Ok(version_json)
}

pub async fn run_processors(
    client_path: &PathBuf,
    libraries_dir: &PathBuf,
    instance_path: &PathBuf,
    java_path: &PathBuf,
    package: ForgePackage,
) -> Result<()> {
    if let Some(processors) = &package.processors {
        if let Some(mut data) = package.data {
            processor_rules! {
                &mut data;
                "SIDE":
                    client => "client",
                    server => "";
                "MINECRAFT_JAR" :
                    client => client_path.to_string_lossy(),
                    server => "";
                "MINECRAFT_VERSION":
                    client => package.inherits_from,
                    server => "";
                "ROOT":
                    client => instance_path.to_string_lossy(),
                    server => "";
                "LIBRARY_DIR":
                    client => libraries_dir.to_string_lossy(),
                    server => "";
            }

            let total_length = processors.len();

            for (index, processor) in processors.iter().enumerate() {
                if let Some(sides) = &processor.sides {
                    if !sides.contains(&String::from("client")) {
                        continue;
                    }
                }

                let cp = wrap_ref_builder!(cp = processor.classpath.clone() => {
                    cp.push(processor.jar.clone())
                });

                let child = Command::new(java_path)
                    .arg("-cp")
                    .arg(args::get_class_paths_jar(&libraries_dir, &cp)?)
                    .arg(
                        args::get_processor_main_class(args::get_lib_path(
                            &libraries_dir,
                            &processor.jar,
                            false,
                        )?)
                        .await?
                        .ok_or_else(|| {
                            crate::error::Error::UnknownError(format!(
                                "Could not find processor main class for {}",
                                processor.jar
                            ))
                        })?,
                    )
                    .args(args::get_processor_arguments(
                        &libraries_dir,
                        &processor.args,
                        &data,
                    )?)
                    .output()
                    .await
                    .map_err(|e| crate::error::Error::UnknownError(java_path.display().to_string()))
                    .map_err(|err| {
                        crate::error::Error::UnknownError(
                            format!("Error running processor: {err}",),
                        )
                    })?;

                if !child.status.success() {
                    return Err(crate::error::Error::UnknownError(format!(
                        "Processor error: {}",
                        String::from_utf8_lossy(&child.stderr)
                    )));
                }
            }
        }
    }
    Ok(())
}

pub fn get_path_from_artifact(artifact: &str) -> Result<String> {
    let name_items = artifact.split(':').collect::<Vec<&str>>();

    let package = name_items.first().ok_or_else(|| {
        Error::UnknownError(format!("Unable to find package for library {}", &artifact))
    })?;
    let name = name_items.get(1).ok_or_else(|| {
        Error::UnknownError(format!("Unable to find name for library {}", &artifact))
    })?;

    if name_items.len() == 3 {
        let version_ext = name_items
            .get(2)
            .ok_or_else(|| {
                Error::UnknownError(format!("Unable to find version for library {}", &artifact))
            })?
            .split('@')
            .collect::<Vec<&str>>();
        let version = version_ext.first().ok_or_else(|| {
            Error::UnknownError(format!("Unable to find version for library {}", &artifact))
        })?;
        let ext = version_ext.get(1);

        Ok(format!(
            "{}/{}/{}/{}-{}.{}",
            package.replace('.', "/"),
            name,
            version,
            name,
            version,
            ext.unwrap_or(&"jar")
        ))
    } else {
        let version = name_items.get(2).ok_or_else(|| {
            Error::UnknownError(format!("Unable to find version for library {}", &artifact))
        })?;

        let data_ext = name_items
            .get(3)
            .ok_or_else(|| {
                Error::UnknownError(format!("Unable to find data for library {}", &artifact))
            })?
            .split('@')
            .collect::<Vec<&str>>();
        let data = data_ext.first().ok_or_else(|| {
            Error::UnknownError(format!("Unable to find data for library {}", &artifact))
        })?;
        let ext = data_ext.get(1);

        Ok(format!(
            "{}/{}/{}/{}-{}-{}.{}",
            package.replace('.', "/"),
            name,
            version,
            name,
            version,
            data,
            ext.unwrap_or(&"jar")
        ))
    }
}
