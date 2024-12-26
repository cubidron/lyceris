pub mod auth;
pub mod error;
pub mod http;
pub mod json;
pub mod macros;
pub mod minecraft;
pub mod util;

use minecraft::{config::Config, emitter::Emitter, install::install, loader::{fabric::Fabric, forge::Forge, quilt::Quilt}};

use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[tokio::test]
async fn test() {
    let config = Config {
        game_dir: "C:\\Users\\batuh\\AppData\\Roaming\\.test".into(),
        version: "1.16.5".to_string(),
        authentication: auth::AuthMethod::Offline {
            username: "Miate".to_string(),
            uuid: "4c4ae28c-16c8-49f4-92a6-8d21e0d8b4a0".to_string(),
        },
        memory: None,
        java_version: None,
        version_name: None,
        loader: Some(Quilt("0.27.1".into())),
        runtime_dir: Some("C:\\Users\\batuh\\AppData\\Roaming\\.minecraft\\runtimes".into()),
        custom_args: vec![],
        custom_java_args: vec![],
    };
    let emitter = Emitter::default();
    emitter.on("single_download_progress", |(path, current, max): (String, u64, u64)| {
        println!("{}-{}/{}", path, current, max);
    }).await;
    emitter.on(
        "multiple_download_progress",
        |(path, current, max): (String, u64, u64)| {
            println!("{}-{}/{}", path, current, max);
        },
    ).await;
    emitter.on("console", |line: String| {
        println!("Line: {}", line);
    }).await;

    install(&config, Some(&emitter)).await.unwrap();

    let mut child = minecraft::launch::launch(&config, Some(&emitter)).await.unwrap();
    child.wait().await.unwrap();

}
