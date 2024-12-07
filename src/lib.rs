//! A Core Library of Cardinal Systems.
//! It only includes minecraft launcher library for now.
//!
//! DOWNLOAD PARALLELISM IS NOT IMPLEMENTED YET.
//! ERROR HANDLING IS NOT IMPROVISED YET.
#![allow(unused)]

pub mod error;
pub mod minecraft;
pub mod network;
mod prelude;
pub mod reporter;
pub mod utils;

#[macro_use]
extern crate rust_i18n;

i18n!("locales");

pub fn set_locale(locale: &str) {
    rust_i18n::set_locale(locale);
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use std::sync::mpsc;

    use crate::minecraft::auth::online::{self, Online};
    use crate::minecraft::custom::fabric::Fabric;
    use crate::minecraft::custom::forge::{self, Forge};
    use crate::minecraft::custom::optifine::OptiFine;
    use crate::minecraft::custom::quilt::Quilt;
    use crate::minecraft::version::Custom;
    use crate::minecraft::{auth, Config};
    use crate::network::post;
    use crate::{
        minecraft::{version::MinecraftVersion, Instance},
        reporter::Reporter,
    };
    use oauth2::basic::BasicClient;
    use oauth2::reqwest::async_http_client;
    use oauth2::{
        AuthType, AuthUrl, AuthorizationCode, ClientId, CsrfToken, DeviceAuthorizationUrl,
        PkceCodeChallenge, RedirectUrl, Scope, StandardDeviceAuthorizationResponse, TokenResponse,
        TokenUrl,
    };
    use reqwest::{Body, Client, Url};
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};
    use tokio::io::AsyncBufReadExt;
    use tokio::net::TcpListener;
    use tokio::runtime::Runtime;
    use tokio::{
        io::{AsyncWriteExt, BufReader},
        test,
    };

    #[derive(Clone)]
    struct TestReporter {}

    impl Reporter for TestReporter {
        fn send(&self, case: crate::reporter::Case) {
            //println!("{:?}", case);
        }
    }

    // #[test]
    // async fn test_launch() {
    //     let test =
    //         Online::authenticate("M.C514_BAY.2.U.413e6719-12c4-33ca-32a7-f7eaf6065052".to_string())
    //             .await
    //             .unwrap();

    //     println!("{:?}", test);
    // }
    // #[test]
    // async fn retrieve_code() {
    //     println!("{:?}", Online::create_link());
    // }

    // #[tokio::test]
    // async fn test_server() {
    //     start_server(
    //         "http://45.141.150.191:3000".to_string(),
    //         "45.141.150.191".to_string(),
    //         7000,
    //         "123".to_string(),
    //         "localhost".to_string(),
    //         25565,
    //         "mc".to_string(),
    //     )
    //     .await.unwrap();
    // }

    // #[tokio::test]
    // async fn launch_game() {
    //     let mut instance = Instance::new();

    //     instance
    //         .launch(
    //             None::<()>,
    //             Config {
    //                 ..Config::default()
    //             },
    //             |e| println!("{:?}", e),
    //         )
    //         .await
    //         .unwrap();

    //     loop {
    //         println!("Polling check");
    //         if let Some(status) = instance.poll() {
    //             println!("Closed!");
    //         }

    //         tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    //     }
    // }

    #[tokio::test]
    pub async fn test_forge() {
        let mut forge = Forge::new((1, 20, None), "46.0.14".to_string());

        let mut instance = Instance::new();

        instance
            .launch(
                TestReporter{},
                Config {
                    version: MinecraftVersion::Custom(Custom::Forge(forge)),
                    java_version: crate::minecraft::java::JavaVersion::Delta,
                    ..Config::default()
                },
                |e| println!("{}", e),
            )
            .await
            .unwrap();


                loop {
            println!("Polling check");
            if let Some(status) = instance.poll() {
                println!("Closed!");
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        }
    }
}
