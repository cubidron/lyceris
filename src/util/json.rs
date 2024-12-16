use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};

use super::error::UtilError;

/// Asynchronously reads a JSON file and deserializes its contents into a specified type.
///
/// # Type Parameters
///
/// - `T`: The type to which the JSON data will be deserialized. This type must implement the
///   `DeserializeOwned` trait from Serde, which allows for deserialization of owned types.
///
/// # Parameters
///
/// - `file_path`: A `String` representing the path to the JSON file to be read.
///
/// # Returns
///
/// This function returns a `Result<T, UtilError>`, where:
/// - `Ok(T)`: Contains the deserialized data of type `T` if the operation is successful.
/// - `Err(UtilError)`: Contains an error of type `UtilError` if the operation fails. This can occur
///   due to issues such as file not found, read errors, or JSON deserialization errors.
///
/// # Errors
///
/// The function may return an error if:
/// - The specified file does not exist or cannot be opened.
/// - There is an error reading the file's contents.
/// - The contents of the file cannot be deserialized into the specified type `T`.
pub async fn read_json<T: DeserializeOwned, P: AsRef<Path>>(file_path: P) -> Result<T, UtilError> {
    let mut file = File::open(file_path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(serde_json::from_str(&contents)?)
}

/// Asynchronously writes a JSON file and deserializes its contents into a specified type.
///
/// # Type Parameters
///
/// - `T`: The type to which the JSON data will be deserialized. This type must implement the
///   `DeserializeOwned` trait from Serde, which allows for deserialization of owned types.
///
/// # Parameters
///
/// - `file_path`: A `AsRef<Path>` representing the path to the JSON file to be read.
/// - `value`: A value that serializable.
///
/// # Returns
///
/// This function returns a `Result<T, UtilError>`, where:
/// - `Ok(())`: If successfull.
/// - `Err(UtilError)`: Contains an error of type `UtilError` if the operation fails. This can occur
///   due to issues such as file not found, read errors, or JSON deserialization errors.
///
/// # Errors
///
/// The function may return an error if:
/// - The specified file does not exist or cannot be opened.
/// - There is an error reading the file's contents.
/// - The contents of the file cannot be deserialized into the specified type `T`.
pub async fn write_json<T: Serialize, P: AsRef<Path>>(
    file_path: P,
    value: &T,
) -> Result<(), UtilError> {
    let json_string = serde_json::to_string(value)?;
    if let Some(parent) = file_path.as_ref().parent() {
        if !parent.is_dir() {
            create_dir_all(parent).await?;
        }
    }
    let mut file = File::create(file_path).await?;
    file.write_all(json_string.as_bytes()).await?;
    Ok(())
}
