use std::{ops::Index,collections::HashMap};
use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize)]
pub struct JavaRuntime {
    #[serde(rename = "gamecore")]
    pub gamecore: Gamecore,

    #[serde(rename = "linux")]
    pub linux: Gamecore,

    #[serde(rename = "linux-i386")]
    pub linux_i386: Gamecore,

    #[serde(rename = "mac-os")]
    pub mac_os: Gamecore,

    #[serde(rename = "mac-os-arm64")]
    pub mac_os_arm64: Gamecore,

    #[serde(rename = "windows-arm64")]
    pub windows_arm64: Gamecore,

    #[serde(rename = "windows-x64")]
    pub windows_x64: Gamecore,

    #[serde(rename = "windows-x86")]
    pub windows_x86: Gamecore,
}

#[derive(Serialize, Deserialize)]
pub struct Gamecore {
    #[serde(rename = "java-runtime-alpha")]
    pub java_runtime_alpha: Vec<JavaRuntimeVersion>,

    #[serde(rename = "java-runtime-beta")]
    pub java_runtime_beta: Vec<JavaRuntimeVersion>,

    #[serde(rename = "java-runtime-gamma")]
    pub java_runtime_gamma: Vec<JavaRuntimeVersion>,

    #[serde(rename = "java-runtime-gamma-snapshot")]
    pub java_runtime_gamma_snapshot: Vec<JavaRuntimeVersion>,

    #[serde(rename = "java-runtime-delta")]
    pub java_runtime_delta : Vec<JavaRuntimeVersion>,

    #[serde(rename = "jre-legacy")]
    pub jre_legacy: Vec<JavaRuntimeVersion>,

    #[serde(rename = "minecraft-java-exe")]
    pub minecraft_java_exe: Vec<JavaRuntimeVersion>,
}

#[derive(Serialize, Deserialize)]
pub struct JavaRuntimeVersion {
    #[serde(rename = "availability")]
    pub availability: Availability,

    #[serde(rename = "manifest")]
    pub manifest: Manifest,

    #[serde(rename = "version")]
    pub version: Version,
}

#[derive(Serialize, Deserialize)]
pub struct Availability {
    #[serde(rename = "group")]
    pub group: i64,

    #[serde(rename = "progress")]
    pub progress: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    #[serde(rename = "sha1")]
    pub sha1: String,

    #[serde(rename = "size")]
    pub size: i64,

    #[serde(rename = "url")]
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Version {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "released")]
    pub released: String,
}

#[derive(Serialize, Deserialize)]
pub struct JavaManifest{
    pub files : HashMap<String,File>,

}

#[derive(Serialize, Deserialize)]
pub struct File{
    #[serde(rename = "type")]
    pub type_ : String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads : Option<Downloads>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable : Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target : Option<String>

}
#[derive(Serialize, Deserialize)]
pub struct Downloads{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lzma : Option<Manifest>,
    pub raw : Manifest
}

impl Index<&str> for JavaRuntime{
    type Output = Gamecore;
    fn index(&self, index: &str) -> &Self::Output {
        match index {
            "gamecore" => &self.gamecore,
            "linux" => &self.linux,
            "linux-x64" => &self.linux,
            "linux-i386" => &self.linux_i386,
            "mac-os" => &self.mac_os,
            "mac-os-arm64" => &self.mac_os_arm64,
            "windows-arm64" => &self.windows_arm64,
            "windows-x64" => &self.windows_x64,
            "windows-x86" => &self.windows_x86,
            _ => panic!("Unsupported OS"),
        }
    }
}

impl Index<&str> for Gamecore{
    type Output = JavaRuntimeVersion;

    fn index(&self, index: &str) -> &Self::Output {
        match index {
            "java-runtime-alpha" => &self.java_runtime_alpha[0],
            "java-runtime-beta" => &self.java_runtime_beta[0],
            "java-runtime-gamma" => &self.java_runtime_gamma[0],
            "java-runtime-gamma-snapshot" => &self.java_runtime_gamma_snapshot[0],
            "java-runtime-delta" => &self.java_runtime_delta[0],
            "jre-legacy" => &self.jre_legacy[0],
            _ => panic!("Unsupported Java Type"),
        }
    }
}