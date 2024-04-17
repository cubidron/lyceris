use std::collections::HashMap;
use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize,Clone,Debug,Default)]
pub struct VersionManifest {
    #[serde(rename = "latest")]
    pub latest: Latest,

    #[serde(rename = "versions")]
    pub versions: Vec<Version>,
}

#[derive(Serialize, Deserialize,Clone,Debug,Default)]
pub struct Latest {
    #[serde(rename = "release")]
    pub release: String,

    #[serde(rename = "snapshot")]
    pub snapshot: String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Version {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "type")]
    pub version_type: Type,

    #[serde(rename = "url")]
    pub url: String,

    #[serde(rename = "time")]
    pub time: String,

    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq)]
pub enum Type {
    #[serde(rename = "old_alpha")]
    OldAlpha,

    #[serde(rename = "old_beta")]
    OldBeta,

    #[serde(rename = "release")]
    Release,

    #[serde(rename = "snapshot")]
    Snapshot,

    #[serde(rename = "custom")]
    Custom
}

#[derive(Serialize, Deserialize,Clone,Debug,Default)]
pub struct Package {
    #[serde(rename = "arguments")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Arguments>,

    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,

    #[serde(rename = "inheritsFrom")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherits_from : Option<String>,

    #[serde(rename = "assets")]
    pub assets: String,

    #[serde(rename = "complianceLevel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compliance_level: Option<i64>,

    #[serde(rename = "downloads")]
    pub downloads: PackageDownloads,

    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "javaVersion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_version: Option<JavaVersion>,

    #[serde(rename = "libraries")]
    pub libraries: Vec<Library>,

    #[serde(rename = "logging")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Logging>,

    #[serde(rename = "mainClass")]
    pub main_class: String,

    #[serde(rename = "minecraftArguments")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments : Option<String>,
    
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: i64,

    #[serde(rename = "releaseTime")]
    pub release_time: String,

    #[serde(rename = "time")]
    pub time: String,

    #[serde(rename = "type")]
    pub package_type: String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Arguments {
    #[serde(rename = "game")]
    pub game: Vec<GameElement>,

    #[serde(rename = "jvm")]
    pub jvm: Vec<JvmElement>,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct GameClass {
    #[serde(rename = "rules")]
    pub rules: Vec<GameRule>,

    #[serde(rename = "value")]
    pub value: Value,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct GameRule {
    #[serde(rename = "action")]
    pub action: Action,

    #[serde(rename = "features")]
    pub features: Features,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Features {
    #[serde(rename = "is_demo_user")]
    pub is_demo_user: Option<bool>,

    #[serde(rename = "has_custom_resolution")]
    pub has_custom_resolution: Option<bool>,

    #[serde(rename = "has_quick_plays_support")]
    pub has_quick_plays_support: Option<bool>,

    #[serde(rename = "is_quick_play_singleplayer")]
    pub is_quick_play_singleplayer: Option<bool>,

    #[serde(rename = "is_quick_play_multiplayer")]
    pub is_quick_play_multiplayer: Option<bool>,

    #[serde(rename = "is_quick_play_realms")]
    pub is_quick_play_realms: Option<bool>,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct JvmClass {
    #[serde(rename = "rules")]
    pub rules: Vec<JvmRule>,

    #[serde(rename = "value")]
    pub value: Value,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct JvmRule {
    #[serde(rename = "action")]
    pub action: Action,

    #[serde(rename = "os")]
    pub os: PurpleOs,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct PurpleOs {
    #[serde(rename = "name")]
    pub name: Option<Name>,

    #[serde(rename = "arch")]
    pub arch: Option<String>,
}

#[derive(Serialize, Deserialize,Clone,Debug,Default)]
pub struct AssetIndex {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "sha1")]
    pub sha1: String,

    #[serde(rename = "size")]
    pub size: i64,

    #[serde(rename = "totalSize")]
    pub total_size: Option<i64>,

    #[serde(rename = "url")]
    pub url: String,
}

#[derive(Serialize, Deserialize,Clone,Debug,Default)]
pub struct PackageDownloads {
    #[serde(rename = "client")]
    pub client: ClientMappingsClass,

    #[serde(rename = "client_mappings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_mappings: Option<ClientMappingsClass>,

    #[serde(rename = "server")]
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub server: Option<ClientMappingsClass>,

    #[serde(rename = "server_mappings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_mappings: Option<ClientMappingsClass>,
}

#[derive(Serialize, Deserialize,Clone,Debug,Default)]
pub struct ClientMappingsClass {
    #[serde(rename = "sha1")]
    pub sha1: String,

    #[serde(rename = "size")]
    pub size: i64,

    #[serde(rename = "url")]
   pub  url: String,

    #[serde(rename = "path")]
    #[serde(skip_serializing_if= "Option::is_none")]
    pub path: Option<String>,
}
#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct ClientMappingsClass2 {
    #[serde(rename = "sha1")]
    pub sha1: String,

    #[serde(rename = "size")]
    pub size: i64,

    #[serde(rename = "url")]
   pub  url: String,

    #[serde(rename = "path")]
    pub path: String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct JavaVersion {
    #[serde(rename = "component")]
    pub component: String,

    #[serde(rename = "majorVersion")]
    pub major_version: i64,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Library {
    #[serde(rename = "downloads")]
    pub downloads: LibraryDownloads,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "rules")]
    pub rules: Option<Vec<LibraryRule>>,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct LibraryDownloads {
    #[serde(rename = "artifact")]
    pub artifact: Option<ClientMappingsClass2>,
    #[serde(rename = "classifiers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifiers : Option<Classifiers>
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Classifiers{
    #[serde(rename = "natives-linux")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives_linux : Option<ClientMappingsClass2>,

    #[serde(rename = "natives-windows")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives_windows : Option<ClientMappingsClass2>,

    #[serde(rename = "natives-macos")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives_macos : Option<ClientMappingsClass2>,

    #[serde(rename = "natives-windows-64")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives_windows_64 : Option<ClientMappingsClass2>,

    #[serde(rename = "natives-windows-32")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives_windows_32 : Option<ClientMappingsClass2>,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct LibraryRule {
    #[serde(rename = "action")]
    pub action: Action,

    #[serde(rename = "os")]
    pub os: Option<FluffyOs>,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct FluffyOs {
    #[serde(rename = "name")]
    pub name: Name,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Logging {
    #[serde(rename = "client")]
    pub client: LoggingClient,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct LoggingClient {
    #[serde(rename = "argument")]
    pub argument: String,

    #[serde(rename = "file")]
    pub file: AssetIndex,

    #[serde(rename = "type")]
    pub client_type: String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
#[serde(untagged)]
pub enum GameElement {
    GameClass(GameClass),

    String(String),
}

#[derive(Serialize, Deserialize,Clone,Debug)]
#[serde(untagged)]
pub enum Value {
    String(String),

    StringArray(Vec<String>),
}

#[derive(Serialize, Deserialize,Clone,Debug)]
#[serde(untagged)]
pub enum JvmElement {
    JvmClass(JvmClass),

    String(String),
}

#[derive(Serialize, Deserialize,Clone,PartialEq,Debug)]
pub enum Action {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "disallow")]
    Disallow
}

#[derive(Serialize, Deserialize,Clone,PartialEq,Debug)]
pub enum Name {
    #[serde(rename = "linux")]
    Linux,

    #[serde(rename = "osx")]
    Osx,

    #[serde(rename = "windows")]
    Windows,
}

#[derive(Serialize, Deserialize,Clone)]
pub struct Index {
    #[serde(rename = "objects")]
    pub objects: HashMap<String, Object>,

    #[serde(rename = "virtual")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_ : Option<bool>,

    #[serde(rename = "map_to_resources")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_to_resources : Option<bool>
}

#[derive(Serialize, Deserialize,Clone)]
pub struct Object {
    #[serde(rename = "hash")]
    pub hash: String,

    #[serde(rename = "size")]
    pub size: i64,
}

impl Iterator for Name{
    type Item = &'static str;
    fn next(&mut self) -> Option<Self::Item>{
        match self{
            Name::Linux => Some("linux"),
            Name::Osx => Some("osx"),
            Name::Windows => Some("windows")
        }
    }
}

impl JavaVersion{
    pub fn convert(&self) -> super::java::JavaVersion{
        match self.component.as_str(){
            "java-runtime-gamma" => super::java::JavaVersion::Gamma,
            "jre-legacy"=> super::java::JavaVersion::Legacy,
            "java-runtime-beta" => super::java::JavaVersion::Beta,
            "java-runtime-alpha"=> super::java::JavaVersion::Alpha,
            _=>super::java::JavaVersion::Gamma
        }
    }
}