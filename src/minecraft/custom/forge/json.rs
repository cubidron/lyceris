use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::minecraft::{self, json::LibraryDownloads};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "inheritsFrom")]
    pub inherits_from: String,

    #[serde(rename = "releaseTime")]
    pub release_time: String,

    #[serde(rename = "time")]
    pub time: String,

    #[serde(rename = "type")]
    pub package_type: String,

    #[serde(rename = "mainClass")]
    pub main_class: String,

    #[serde(rename = "arguments")]
    pub arguments: Arguments,

    #[serde(rename = "libraries")]
    pub libraries: Vec<Library>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data : Option<HashMap<String, SidedDataEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processors: Option<Vec<Processor>>
}

#[derive(Serialize, Debug,Deserialize, Clone)]
pub struct InstallerProfile {
    pub data: HashMap<String, SidedDataEntry>,
    pub libraries: Vec<Library>,
    pub processors: Vec<Processor>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Arguments {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "game")]
    pub game: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "jvm")]
    pub jvm: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Library {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The URL to the repository where the library can be downloaded
    pub url: Option<String>,

    #[serde(rename = "md5")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,

    #[serde(rename = "sha1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    #[serde(rename = "sha256")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

    #[serde(rename = "sha512")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha512: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<LibraryDownloads>,

    #[serde(rename = "size")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(default = "default_include_in_classpath")]
    pub exclude: bool
}

impl From<&Library> for minecraft::json::Library {
    fn from(value: &Library) -> Self {
        Self {
            downloads: value.downloads.clone().unwrap(),
            name: value.name.clone(),
            rules: None
        }
    }
}

fn default_include_in_classpath() -> bool {
    false
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SidedDataEntry {
    /// The value on the client
    pub client: String,
    /// The value on the server
    pub server: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Processor {
    /// Maven coordinates for the JAR library of this processor.
    pub jar: String,
    /// Maven coordinates for all the libraries that must be included in classpath when running this processor.
    pub classpath: Vec<String>,
    /// Arguments for this processor.
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Represents a map of outputs. Keys and values can be data values
    pub outputs: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Which sides this processor shall be ran on.
    /// Valid values: client, server, extract
    pub sides: Option<Vec<String>>,
}