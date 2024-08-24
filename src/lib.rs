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
    use std::path::PathBuf;

    use crate::minecraft::custom::fabric::Fabric;
    use crate::minecraft::custom::quilt::Quilt;
    use crate::minecraft::version::Custom;
    use crate::minecraft::Config;
    use crate::network::post;
    use crate::{
        minecraft::{version::MinecraftVersion, Instance},
        reporter::Reporter,
    };
    use reqwest::Body;
    use serde_json::{json, Value};
    use tokio::io::AsyncBufReadExt;
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
        let mut launcher: Instance<TestReporter> = Instance::new(
            Config {
                version: MinecraftVersion::Custom(Custom::Quilt(Quilt::new(
                    (1, 20, Some(4)),
                    "0.24.0".to_string(),
                ))),
                //custom_java_args: vec!["-XstartOnFirstThread".to_string()],
                ..Config::default()
            },
            Some(TestReporter {}),
        );
        let mut p = launcher.launch().await.unwrap();
        let stdout = p.stdout.take().expect("no stdout");

        let mut lines = BufReader::new(stdout).lines();
        while let Some(line) = lines.next_line().await.unwrap() {
            println!("{}", line);
        }
    }
}
