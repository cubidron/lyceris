use serde::{Deserialize, Serialize};

use super::vanilla::{Element, LibraryDownloads};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomMeta {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherits_from: Option<String>,
    pub release_time: String,
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    pub main_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Arguments>,
    pub libraries: Vec<Library>,
}

#[derive(Serialize, Deserialize)]
pub struct Arguments {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game: Option<Vec<Element>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jvm: Option<Vec<Element>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Library {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha512: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<LibraryDownloads>,
}
