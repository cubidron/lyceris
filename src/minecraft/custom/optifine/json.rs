use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "inheritsFrom")]
    pub inherits_from: String,

    #[serde(rename = "time")]
    pub time: String,

    #[serde(rename = "releaseTime")]
    pub release_time: String,

    #[serde(rename = "libraries")]
    pub libraries: Vec<Library>,

    #[serde(rename = "mainClass")]
    pub main_class: String,

    #[serde(rename = "arguments")]
    pub arguments: Arguments,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Arguments {
    #[serde(rename = "game")]
    pub game: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Library {
    #[serde(rename = "name")]
    pub name: String,
}
