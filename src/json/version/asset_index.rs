use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#virtual: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_to_resources: Option<bool>
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub hash: String,
    pub size: u64
}