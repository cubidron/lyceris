#![allow(unused_imports)]

use std::f32::consts::E;

use event_emitter_rs::EventEmitter;
use minecraft::{
    install::install,
    launch::{launch, Config},
    loaders::{fabric::Fabric, quilt::Quilt},
};
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
        version: "1.21.4".to_string(),
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

    // emitter.on(
    //     "multiple_download_progress",
    //     |(current, max): (u64, u64)| {
    //         println!("{}/{}", current, max);
    //     },
    // );

    emitter.on("single_download_progress", |(current, max): (u64, u64)| {
        println!("{}/{}", current, max);
    });

    install(&config, Some(&mut emitter)).await.unwrap();

    let mut cmd = launch(&config, Some(&mut emitter)).await.unwrap();

    cmd.spawn().unwrap().wait_with_output().await.unwrap();
}
