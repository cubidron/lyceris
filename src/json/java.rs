use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type JavaManifest = HashMap<String, HashMap<String, Vec<Gamecore>>>;

#[derive(Serialize, Deserialize)]
pub struct Gamecore {
    pub availability: Availability,
    pub manifest: FileMap,
    pub version: Version,
}

#[derive(Serialize, Deserialize)]
pub struct Availability {
    pub group: u32,
    pub progress: u16,
}

#[derive(Serialize, Deserialize)]
pub struct FileMap {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Version {
    pub name: String,
    pub released: String,
}

#[derive(Serialize, Deserialize)]
pub struct JavaFileManifest {
    pub files: HashMap<String, File>,
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<Downloads>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Downloads {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lzma: Option<FileMap>,
    pub raw: FileMap,
}
