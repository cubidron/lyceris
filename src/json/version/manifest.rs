use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct VersionManifest {
    #[serde(rename = "latest")]
    pub latest: Latest,

    #[serde(rename = "versions")]
    pub versions: Vec<Version>,
}

#[derive(Serialize, Deserialize)]
pub struct Latest {
    #[serde(rename = "release")]
    pub release: String,

    #[serde(rename = "snapshot")]
    pub snapshot: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    pub r#type: Type,
    pub url: String,
    pub time: String,
    pub release_time: String,
}

#[derive(Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "old_alpha")]
    OldAlpha,

    #[serde(rename = "old_beta")]
    OldBeta,

    #[serde(rename = "release")]
    Release,

    #[serde(rename = "snapshot")]
    Snapshot,
}
