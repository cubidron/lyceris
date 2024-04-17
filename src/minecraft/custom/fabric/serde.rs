use serde::{Serialize,Deserialize};
use serde_json::Value;


#[derive(Serialize, Deserialize,Debug,Clone)]
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
}

#[derive(Serialize, Deserialize,Debug,Clone)]
pub struct Arguments {
    #[serde(rename = "game")]
    pub game: Vec<Option<Value>>,

    #[serde(rename = "jvm")]
    pub jvm: Vec<String>,
}

#[derive(Serialize, Deserialize,Debug,Clone)]
pub struct Library {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "url")]
    pub url: String,

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

    #[serde(rename = "size")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
}

pub type LoaderList = Vec<Loader>;
pub type VersionList = Vec<Version>;

#[derive(Serialize, Deserialize)]
pub struct Loader {
    #[serde(rename = "separator")]
    pub separator: Separator,

    #[serde(rename = "build")]
    pub build: i64,

    #[serde(rename = "maven")]
    pub maven: String,

    #[serde(rename = "version")]
    pub version: String,

    #[serde(rename = "stable")]
    pub stable: bool,
}

#[derive(Serialize, Deserialize)]
pub enum Separator {
    #[serde(rename = "+build.")]
    Build,

    #[serde(rename = ".")]
    Empty,
}

#[derive(Serialize, Deserialize)]
pub struct Version {
    #[serde(rename = "version")]
    pub version: String,

    #[serde(rename = "stable")]
    pub stable: bool,
}

