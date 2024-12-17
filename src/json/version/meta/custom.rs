use serde::{Deserialize, Serialize};

use super::vanilla::Element;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomMeta {
    pub id: String,
    pub inherits_from: String,
    pub release_time: String,
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_type: Option<String>,
    pub main_class: String,
    pub arguments: Arguments,
    pub libraries: Vec<Library>,
}

#[derive(Serialize, Deserialize)]
pub struct Arguments {
    pub game: Vec<Element>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jvm: Option<Vec<Element>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Library {
    pub name: String,
    pub url: String,

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
}
