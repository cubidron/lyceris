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
    use std::io::Write;
    use std::net::{SocketAddr};
    use std::path::PathBuf;
    use std::sync::mpsc;

    use crate::minecraft::auth::online::{self, Online};
    use crate::minecraft::custom::fabric::Fabric;
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
    use oauth2::{AuthType, AuthUrl, AuthorizationCode, ClientId, CsrfToken, DeviceAuthorizationUrl, PkceCodeChallenge, RedirectUrl, Scope, StandardDeviceAuthorizationResponse, TokenResponse, TokenUrl};
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
            println!("{:?}", case);
        }
    }

    #[test]
    async fn test_launch() {
        let test = Online::authenticate("M.C514_BAY.2.U.413e6719-12c4-33ca-32a7-f7eaf6065052".to_string()).await.unwrap();

        println!("{:?}", test);
    }
    #[test]
    async fn retrieve_code() {
        println!("{:?}", Online::create_link());
    }
    #[test]
    async fn launch_game() {
        let mut instance = Instance::new();

        instance.launch(TestReporter{}, Config{
            authentication: auth::AuthMethod::Online(online::Online::authenticate("M.C531_SN1.2.U.3e231200-c604-773e-3a83-bd0da9400088".to_string()).await.unwrap()), ..Config::default()
        }, |e| println!("{:?}", e)).await.unwrap();
    }
}
