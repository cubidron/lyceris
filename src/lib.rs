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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tokio::io::AsyncBufReadExt;
    use tokio::{
        io::{AsyncWriteExt, BufReader},
        test,
    };

    use crate::{
        minecraft::{version::MinecraftVersion, Launcher},
        reporter::Reporter,
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
        let mut launcher: Launcher = Launcher::new(
            PathBuf::from("../game"),
            MinecraftVersion::Release((1, 20, Some(1))),
        );
        let mut p = launcher.launch().await.unwrap();
        let stdout = p.stdout.take().expect("no stdout");

        let mut lines = BufReader::new(stdout).lines();
        while let Some(line) = lines.next_line().await.unwrap() {
            println!("{}", line)
        }
    }
}
