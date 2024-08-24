use std::io::Write;
use std::path::{Path, PathBuf};

use futures_util::{Future, StreamExt, TryFutureExt};
use once_cell::sync::Lazy;
use reqwest::{Body, Client, ClientBuilder, IntoUrl, Response, Url};
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
    ClientBuilder::new()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .expect("Couldn't create the network client.")
});

/// Sends a GET request to the given URL.
///
/// Retries the request if it fails with the given amount of retries (default = 3).
pub async fn get(url: impl IntoUrl) -> Result<Response> {
    Ok(retry(|| CLIENT.get(url.as_str()).header("Cache-Control", "no-cache, no-store, must-revalidate").send(), reqwest::Result::is_ok).await?)
}

/// Sends a POST request to the given URL with payload.
/// 
pub async fn post<P : DeserializeOwned + ToString>(url: impl IntoUrl, payload : P) -> Result<Response>{
    let payload = Body::from(payload.to_string());
    Ok(CLIENT.post(url.as_str()).body(payload).send().await?)
}

/// Sends a GET request to the given URL and returns JSON value.
///
/// Retries the request if it fails with the given amount of retries (default = 3).
pub async fn get_json<F: DeserializeOwned>(url: impl IntoUrl) -> Result<F> {
    Ok(get(url).await?.json::<F>().await?)
}

pub async fn download_retry<R: Reporter>(
    url: impl IntoUrl,
    path: &impl AsRef<Path>,
    reporter: &Option<R>,
) -> Result<()> {
    retry(|| download(url.as_str(), path, reporter), Result::is_ok).await
}

/// Download file from the given URL to the destination path.
///
/// Retries the download if it fails with the given amount of retries (default = 3).
pub async fn download<R: Reporter>(
    url: impl IntoUrl,
    path: impl AsRef<Path>,
    reporter: &Option<R>,
) -> Result<()> {
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

    reporter.send(Case::SetMaxSubProgress(total_size as f64));

    while let Some(item) = stream.next().await {
        let chunk = item?;
        temp_file.write_all(&chunk).await?;
        progress += chunk.len() as f64;

        reporter.send(Case::SetSubProgress(progress));
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
