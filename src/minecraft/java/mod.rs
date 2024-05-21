use std::{env, fmt::Display, path::{Path, PathBuf}, process::Command};

use ::serde::Deserialize;

use crate::{network::get_json, prelude::Result, utils::hash_file};

use self::serde::{JavaManifest, JavaRuntime};

mod serde;

#[derive(Debug, PartialEq, PartialOrd,Deserialize)]
pub enum JavaVersion {
    Gamma,
    Alpha,
    Beta,
    Legacy,
}

const JAVA_RUNTIMES : &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

impl Display for JavaVersion{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::Alpha=>{
                write!(f,"java-runtime-alpha")
            },
            Self::Beta=>{
                write!(f,"java-runtime-beta")
            },
            Self::Gamma=>{
                write!(f,"java-runtime-gamma")
            },
            Self::Legacy=>{
                write!(f,"jre-legacy")
            }
        }
    }
}


pub fn get_java_path() -> Option<PathBuf>{
    match Command::new("java").arg("-version").output() {
        Ok(output) => {
            let output = std::str::from_utf8(&output.stderr).unwrap_or("");
            if !output.is_empty() {
                let path = output.split(' ').collect::<Vec<&str>>()[3]
                    .split('=')
                    .collect::<Vec<&str>>()[1];
                
                #[cfg(target_os = "windows")]
                return Some(PathBuf::from(path).join("bin").join("java.exe"))

                // Todo : Other operating systems...
                
            }
            None
        }
        Err(e) => {
            None
        }
    }
}
pub fn detect_java_by_cmd() -> Option<JavaVersion> {
    match Command::new("java").arg("-version").output() {
        Ok(output) => {
            let output = std::str::from_utf8(&output.stderr).unwrap_or("");
            if !output.is_empty() {
                let split = output.split(' ').collect::<Vec<&str>>()[2]
                    .trim_matches('"')
                    .split('.')
                    .collect::<Vec<&str>>();

                let java_version = if split[0] != "1"{
                    split[0]
                }else{
                    split[1]
                };

                match java_version {
                    "17" => return Some(JavaVersion::Gamma),
                    "16" => return Some(JavaVersion::Alpha),
                    "8" => return Some(JavaVersion::Legacy),
                    _ => {
                        return Some(JavaVersion::Gamma);
                    }
                }
            }
            None
        }
        Err(e) => {
            None
        }
    }
}

pub async fn get_manifest_by_version(version: &JavaVersion) -> Result<JavaManifest> {
    let java_runtime: JavaRuntime = get_json::<JavaRuntime>(JAVA_RUNTIMES).await?;

    let os: String = if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "macos") {
        "mac-os".to_string()
    } else {
        panic!("Unsupported OS");
    };

    let arch: String = match env::consts::ARCH {
        "x86" => {
            if os == "linux" {
                "i386".to_string()
            } else {
                "x86".to_string()
            }
        }
        "x86_64" => "x64".to_string(),
        "aarch64" => "arm64".to_string(),
        _ => panic!("Unsupported architecture"),
    };

    let url = &java_runtime[format!("{}-{}", os, arch).as_str()][match version {
        JavaVersion::Gamma => "java-runtime-gamma",
        JavaVersion::Alpha => "java-runtime-alpha",
        JavaVersion::Beta => "java-runtime-beta",
        JavaVersion::Legacy => "jre-legacy",
    }]
    .manifest
    .url;

    get_json::<JavaManifest>(url).await
}


