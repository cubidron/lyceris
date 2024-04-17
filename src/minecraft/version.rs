use crate::network::Network;

use super::{
    custom::fabric::FabricVersion,
    java::JavaVersion,
    serde::{Type, VersionManifest},
    NETWORK,
};

use core::fmt;

// Constant manifest URL of minecraft versions.
pub static VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

// Default version base. Third one is optional because some cases : 1.16, 1.15, ....
pub type MinecraftVersionBase = (u8, u8, Option<u8>);

// Using custom Display trait as an Extension for MinecraftVersionBase
// Because Rust doesn't allow us to use default trait implementations for custom types.
pub trait Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

pub trait ToString {
    fn to_string(&self) -> String;
}

impl Display for MinecraftVersionBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}{}",
            self.0,
            self.1,
            self.2.map_or_else(|| "".to_string(), |v| format!(".{}",v))
        )
    }
}

impl ToString for MinecraftVersionBase {
    fn to_string(&self) -> String {
        // Use the format! macro to create a formatted string
        format!(
            "{}.{}{}",
            self.0,
            self.1,
            self.2.map_or_else(|| "".to_string(), |v| format!(".{}",v))
        )
    }
}
#[derive(Clone)]
pub enum Custom {
    Fabric(FabricVersion),
    OptiFine(u8, u8, Option<u8>),
    Forge(u8, u8, Option<u8>),
}

// Because of the ability to filter version types in the iteration of version manifest
// We use Release, OldBeta, OldAlpha and Snapshot.
#[derive(Clone)]
pub enum MinecraftVersion {
    Release(MinecraftVersionBase),
    OldBeta(String),
    OldAlpha(String),
    Snapshot(String),
    Custom(Custom),
}

// This implementation necessary for filtering.
impl MinecraftVersion {
    pub fn get_type(&self) -> Type {
        match self {
            Self::Release((_, _, _)) => Type::Release,
            Self::OldAlpha(_) => Type::OldAlpha,
            Self::OldBeta(_) => Type::OldBeta,
            Self::Snapshot(_) => Type::Snapshot,
            Self::Custom(_) => Type::Release,
        }
    }
    pub fn get_compatible_java_version(&self) -> JavaVersion {
        match self {
            Self::Release((v, v1, _)) => {
                if *v >= 1 && *v1 >= 16 {
                    JavaVersion::Gamma
                } else {
                    JavaVersion::Legacy
                }
            }
            Self::OldAlpha(_) => JavaVersion::Legacy,
            Self::OldBeta(_) => JavaVersion::Legacy,
            Self::Snapshot(_) => JavaVersion::Legacy, // todo change in the future
            Self::Custom(_) => JavaVersion::Gamma,
        }
    }
    async fn get_latest_version() -> Self {
        if let Ok(manifest) = NETWORK
            .get_json::<VersionManifest>(VERSION_MANIFEST_URL)
            .await
        {
            let version = manifest.latest.release.split('.').collect::<Vec<&str>>();
            Self::Release((
                version[0].parse::<u8>().unwrap(),
                version[1].parse::<u8>().unwrap(),
                if version.len() > 2 {
                    Some(version[2].parse::<u8>().unwrap())
                } else {
                    None
                },
            ))
        } else {
            Self::Release((1, 20, Some(4)))
        }
    }
}

impl fmt::Display for MinecraftVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Release((v, v1, v2)) => {
                if let Some(v2) = v2 {
                    write!(f, "{}.{}.{}", v, v1, v2)
                } else {
                    write!(f, "{}.{}", v, v1)
                }
            }
            Self::OldAlpha(v) => write!(f, "{}", v),
            Self::OldBeta(v) => write!(f, "{}", v),
            Self::Snapshot(v) => write!(f, "{}", v),
            Self::Custom(v) => match v {
                // Todo : change this.
                Custom::Fabric(fabric) => fabric.version.fmt(f),
                Custom::Forge(v, v1, v2) => {
                    write!(f, "forge-{}.{}.{}", v, v1, v2.unwrap_or_default())
                }
                Custom::OptiFine(v, v1, v2) => {
                    write!(f, "optifine-{}.{}.{}", v, v1, v2.unwrap_or_default())
                }
            },
        }
    }
}
