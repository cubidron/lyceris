use event_emitter_rs::EventEmitter;
use futures::{future::join_all, stream, StreamExt};
use reqwest::{get, IntoUrl};
/// A module for downloading files asynchronously.
use std::{path::Path, sync::Arc, time::Duration};
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
    sync::Mutex,
};

use crate::{
    emit,
    util::retry::{self, retry},
};

use super::error::HttpError;

/// Downloads a file from the specified URL and saves it to the given destination.
///
/// This function performs an asynchronous HTTP GET request to the provided URL,
/// streams the response body, and writes the content to a file at the specified
/// destination. It also provides progress updates through a callback function.
///
/// # Parameters
///
/// - `url`: The URL of the file to download. It can be any type that implements
///   the `IntoUrl` trait, such as a string slice or a `String`.
/// - `destination`: A `PathBuf` representing the path where the downloaded file
///   will be saved.
/// - `progression_callback`: A mutable closure that takes two `u64` parameters:
///   the number of bytes downloaded so far and the total size of the file. This
///   callback is called after each chunk of data is written to the file, allowing
///   the caller to track the download progress.
///
/// # Returns
///
/// This function returns a `Result<u64, DownloaderError>`. On success, it returns
/// `Ok(u64)`. If an error occurs during the download process, it returns an
/// `Err` containing a `DownloaderError` that describes the failure.
///
/// # Errors
///
/// The function can fail in several ways, including but not limited to:
/// - Network errors when making the HTTP request.
/// - Non-success HTTP status codes (e.g., 404 Not Found).
/// - Errors when creating or writing to the file.
pub async fn download<P: AsRef<Path>>(
    url: impl IntoUrl,
    destination: P,
    emitter: Option<&Arc<Mutex<EventEmitter>>>,
) -> Result<u64, HttpError> {
    // Send a get request to the given url.
    let response = get(url).await?;
    if !response.status().is_success() {
        return Err(HttpError::Download(response.status().to_string()));
    }

    // Get the total size of the file to use at progression
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    if let Some(parent) = destination.as_ref().parent() {
        if !parent.is_dir() {
            create_dir_all(parent).await?;
        }
    }

    // Create a file to write the downloaded content
    let mut file = File::create(&destination).await?;

    // Stream the response body
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        downloaded += chunk.len() as u64;

        // Write chunk to the file.
        file.write_all(&chunk).await?;

        emit!(
            emitter,
            "single_download_progress",
            (
                destination.as_ref().to_string_lossy().into_owned(),
                downloaded,
                total_size,
            )
        );
    }

    Ok(total_size)
}

/// Downloads multiple files from the specified URLs and saves them to the given destinations.
///
/// This function takes a vector of tuples, where each tuple contains a URL and a destination path.
/// It downloads all files in parallel and provides progress updates through a callback function.
///
/// # Parameters
///
/// - `downloads`: A vector of tuples containing the URLs and their corresponding destination paths.
/// - `progression_callback`: A mutable closure that takes four `u64` parameters:
///   the number of bytes downloaded so far for the current file, the total bytes downloaded so far,
///   the current file index, and the total number of files. This callback is called after each chunk of data
///   is written to the file, allowing the caller to track the download progress.
///
/// # Returns
///
/// This function returns a `Result<(), HttpError>`. On success, it returns `Ok(())`. If an error occurs
/// during the download process, it returns an `Err` containing a `HttpError` that describes the failure.
pub async fn download_multiple<U, P>(
    downloads: Vec<(U, P)>,
    emitter: Option<&Arc<Mutex<EventEmitter>>>,
) -> Result<(), HttpError>
where
    U: IntoUrl + Send,               // URL type that implements IntoUrl
    P: AsRef<Path> + Send + 'static, // Path type
{
    let total_files = downloads.len();
    let total_downloaded = Arc::new(Mutex::new(0));

    let tasks = downloads.into_iter().map(|(url, destination)| {
        let total_downloaded = Arc::clone(&total_downloaded);
        let emitter = emitter.cloned();

        async move {
            // Retry download logic
            retry(
                || async { download(url.as_str(), destination.as_ref(), emitter.as_ref()).await },
                Result::is_ok,
                3,
                Duration::from_secs(3),
            )
            .await?;

            // Update the progress counter
            let mut downloaded = total_downloaded.lock().await;
            *downloaded += 1;

            emit!(
                emitter,
                "multiple_download_progress",
                (
                    destination.as_ref().to_string_lossy().into_owned(),
                    *downloaded as u64,
                    total_files as u64,
                )
            );

            Ok::<(), HttpError>(()) // Explicit return type
        }
    });

    // Collect tasks into FuturesUnordered to handle early exit on error
    let mut futures = stream::FuturesUnordered::new();
    for task in tasks {
        futures.push(task);
    }

    // Poll futures and stop on the first error
    while let Some(result) = futures.next().await {
        result?; // Return early if any task fails
    }

    Ok(())
}
