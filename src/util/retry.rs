/// A module for utility functions, including retry logic.
use std::time::Duration;
use tokio::time::sleep;

use super::error::UtilError;

/// Retries a given asynchronous operation a specified number of times with a delay.
///
/// This function attempts to execute the provided operation up to `max_retries` times.
/// If the operation fails, it waits for the specified `delay` before retrying. If the
/// operation succeeds, the result is returned. If all attempts fail, an error is returned.
///
/// # Parameters
///
/// - `max_retries`: The maximum number of times to retry the operation.
/// - `delay`: The duration to wait between retry attempts.
/// - `operation`: A closure that returns a `Result<T, String>`. This closure is the
///   operation to be retried. It should return `Ok(T)` on success or `Err(String)`
///   on failure.
///
/// # Returns
///
/// This function returns a `Result<T, UtilError>`. On success, it returns `Ok(result)`,
/// where `result` is the successful output of the operation. If all retry attempts fail,
/// it returns an `Err` containing a `UtilError` that describes the failure.
///
/// # Errors
///
/// The function can fail in several ways, including but not limited to:
/// - The operation returns an error after exhausting all retry attempts.
/// - The operation fails to execute due to other unforeseen issues.
pub async fn retry<T, F>(max_retries: u32, delay: Duration, operation: F) -> Result<T, UtilError>
where
    F: Fn() -> Result<T, String>,
{
    let mut attempts = 0;

    while attempts < max_retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts < max_retries {
                    sleep(delay).await;
                } else {
                    return Err(UtilError::Retry(e, max_retries as u8));
                }
            }
        }
    }

    unreachable!()
}
