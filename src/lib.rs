#![allow(unused_imports)]

use std::{f32::consts::E, sync::Arc};

use event_emitter_rs::EventEmitter;
use minecraft::{
    install::install,
    launch::{launch, Config},
    loaders::{fabric::Fabric, quilt::Quilt},
};
use tokio::sync::Mutex;
pub mod auth;
pub mod http;
pub mod json;
pub mod macros;
pub mod minecraft;
pub mod util;

#[tokio::test]
async fn test() {
    let config = Config {
        game_dir: "C:\\Users\\batuh\\AppData\\Roaming\\.test".into(),
        version: "1.20".to_string(),
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

    let mut emitter = EventEmitter::new();

    // emitter.on("single_download_progress", |(path, current, max): (String, u64, u64)| {
    //     println!("{}-{}/{}", path, current, max);
    // });

    emitter.on(
        "multiple_download_progress",
        |(path, current, max): (String, u64, u64)| {
            println!("{}-{}/{}", path, current, max);
        },
    );

    emitter.on("console", |line: String| {
        println!("Line: {}", line);
    });

    let emitter = Arc::new(Mutex::new(emitter));


    install(&config, Some(&emitter)).await.unwrap();

    launch(&config, Some(emitter))
        .await
        .unwrap()
        .wait()
        .await
        .unwrap();
}
