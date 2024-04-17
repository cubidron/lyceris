use std::io::Write;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use once_cell::sync::Lazy;
use reqwest::{Client, ClientBuilder, IntoUrl, Response, Url};
use serde::de::DeserializeOwned;
use tokio::io::AsyncWriteExt;
use tokio::{fs, io};

use crate::prelude::*;
use crate::reporter::Case;
use crate::reporter::Reporter;

pub struct Network<R: Reporter> {
    // Network client for request operations.
    pub client: Client,
    // Retry count if operation fails.
    pub retry_count: usize,
    // All servers for request operations.
    pub servers: Vec<Url>,
    // Reporter for progression.
    pub reporter: Option<R>,
}

impl<R: Reporter> Default for Network<R> {
    fn default() -> Self {
        let client = ClientBuilder::new()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .expect("Couldn't create the network client.");

        Self {
            client,
            retry_count: 3,
            servers: vec![],
            reporter: None,
        }
    }
}

impl<R: Reporter> Network<R> {
    pub fn with_reporter(mut self, reporter: R) -> Self {
        self.reporter = Some(reporter);
        self
    }
}

impl<R: Reporter> Network<R> {
    // That function loops the functions until it reaches to the max.
    async fn retry<A, B: std::future::Future<Output = A>>(
        &self,
        f: impl Fn() -> B,
        handler: impl Fn(&A) -> bool,
    ) -> Result<A> {
        let mut retries = 0;
        loop {
            retries += 1;
            let f = f();
            let r = f.await;
            if handler(&r) || retries >= self.retry_count {
                return Ok(r);
            }
        }
    }

    pub async fn get(&self, url: impl IntoUrl) -> Result<Response> {
        Ok(self
            .retry(
                || self.client.get(url.as_str()).send(),
                reqwest::Result::is_ok,
            )
            .await??)
    }

    pub async fn get_json<T: DeserializeOwned>(&self, url: impl IntoUrl) -> Result<T> {
        Ok(self.get(url.as_str()).await?.json::<T>().await?)
    }

    pub async fn download(&self, url: impl IntoUrl, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let response = self.get(url).await?;

        let total_size = response.content_length().unwrap_or(0);
        fs::create_dir_all(path.parent().unwrap()).await.ok();
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

        self.reporter
            .send(Case::SetMaxSubProgress(total_size as f64));

        while let Some(item) = stream.next().await {
            let chunk = item?;
            temp_file.write_all(&chunk).await?;
            progress += chunk.len() as f64;

            self.reporter.send(Case::SetSubProgress(progress))
        }

        fs::rename(temp_path, path).await?;
        Ok(())
    }

    pub async fn download_with_retry(
        &self,
        url: impl IntoUrl + Clone,
        path: impl AsRef<Path> + Clone,
    ) -> Result<()> {
        self
            .retry(|| self.download(url.clone(), path.clone()), Result::is_ok)
            .await?
    }
}
