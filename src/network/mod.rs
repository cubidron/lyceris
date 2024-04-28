use std::io::Write;
use std::path::{Path, PathBuf};

use futures_util::{Future, StreamExt, TryFutureExt};
use once_cell::sync::Lazy;
use reqwest::{Client, ClientBuilder, IntoUrl, Response, Url};
use serde::de::DeserializeOwned;
use tokio::io::AsyncWriteExt;
use tokio::{fs, io};

use crate::prelude::*;
use crate::reporter::Case;
use crate::reporter::Reporter;

/// Default retry count for operations.
pub const RETRY_COUNT: u8 = 3;

/// Initializes the network client.
///
/// Uses `CARGO_PKG_NAME/CARGO_PKG_VERSION` as user agent.
static CLIENT: Lazy<Client> = Lazy::new(|| {
    let client = ClientBuilder::new()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .expect("Couldn't create the network client.");
    client
});

/// Sends a GET request to the given URL.
///
/// Retries the request if it fails with the given amount of retries (default = 3).
pub async fn get(url: impl IntoUrl) -> Result<Response> {
    R.send(Case::SetSubMessage(format!(
        "Sending GET request to {}",
        url.as_str()
    )));

    Ok(retry(|| CLIENT.get(url.as_str()).send(), reqwest::Result::is_ok).await?)
}
/// Sends a GET request to the given URL and returns JSON value.
/// 
/// Retries the request if it fails with the given amount of retries (default = 3).
pub async fn get_json<F: DeserializeOwned>(url: impl IntoUrl) -> Result<F> {
    Ok(get(url).await?.json::<F>().await?)
}

pub async fn download_retry(url: impl IntoUrl, path: &impl AsRef<Path>) -> Result<()> {
    Ok(retry(|| download(url.as_str(), path), Result::is_ok).await?)
}

/// Download file from the given URL to the destination path.
/// 
/// Retries the download if it fails with the given amount of retries (default = 3).
pub async fn download(url: impl IntoUrl, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let response = get(url).await?;

    let total_size = response.content_length().unwrap_or(0);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    // This is not necessary for now.
    // But necessary for parallelism that will be implemented in the future.
    let temp_path = format!("{}.tmp", path.display());

    let mut temp_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&temp_path)
        .await?;

    let mut progress: f64 = 0f64;
    let mut stream = response.bytes_stream();

    R.send(Case::SetMaxSubProgress(total_size as f64));

    while let Some(item) = stream.next().await {
        let chunk = item?;
        temp_file.write_all(&chunk).await?;
        progress += chunk.len() as f64;

        R.send(Case::SetSubProgress(progress));
    }

    fs::rename(temp_path, path).await?;

    Ok(())
}

/// Method that allows us to retry to call functions without code repeating.
async fn retry<A, B: std::future::Future<Output = A>>(
    f: impl Fn() -> B,
    handler: impl Fn(&A) -> bool,
) -> A {
    let mut retries = 0;
    loop {
        retries += 1;
        let f = f();
        let r: A = f.await;
        if handler(&r) || retries >= RETRY_COUNT {
            return r;
        }
    }
}
