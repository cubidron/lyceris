use std::{collections::HashMap, path::Path};

use crate::prelude::CLASSPATH_SEPERATOR;

use super::{get_path_from_artifact, json::SidedDataEntry};
use std::io::BufReader;
use std::io::BufRead;

use crate::prelude::Result;

use dunce::canonicalize;

pub fn get_class_paths_jar<T: AsRef<str>>(
    libraries_path: &Path,
    libraries: &[T],
) -> Result<String> {
    let cps = libraries
        .iter()
        .map(|library| get_lib_path(libraries_path, library.as_ref(), false))
        .collect::<Result<Vec<_>>>()?;
    
    Ok(cps.join(CLASSPATH_SEPERATOR))
}

pub async fn get_processor_main_class(
    path: String,
) -> Result<Option<String>> {
    let main_class = tokio::task::spawn_blocking(move || {
        let zipfile = std::fs::File::open(&path)
            .map_err(|e| crate::error::Error::UnknownError(e.to_string()))?;
        let mut archive = zip::ZipArchive::new(zipfile).map_err(|_| {
            crate::error::Error::UnknownError(format!(
                "Cannot read processor at {}",
                path
            ))
        })?;

        let file = archive.by_name("META-INF/MANIFEST.MF").map_err(|_| {
            crate::error::Error::UnknownError(format!(
                "Cannot read processor manifest at {}",
                path
            ))
        })?;

        let reader = BufReader::new(file);

        for line in reader.lines() {
            let mut line = line.map_err(|_| crate::error::Error::UnknownError("Error".to_string()))?;
            line.retain(|c| !c.is_whitespace());

            if line.starts_with("Main-Class:") {
                if let Some(class) = line.split(':').nth(1) {
                    return Ok::<std::option::Option<std::string::String>, crate::error::Error>(Some(class.to_string()));
                }
            }
        }

        Ok(None)
    })
    .await??;

    Ok(main_class)
}

pub fn get_lib_path(
    libraries_path: &Path,
    lib: &str,
    allow_not_exist: bool,
) -> Result<String> {
    let mut path = libraries_path.to_path_buf();

    path.push(get_path_from_artifact(lib)?);

    if !path.exists() && allow_not_exist {
        return Ok(path.to_string_lossy().to_string());
    }

    let path = &canonicalize(&path).map_err(|_| {
        crate::error::Error::UnknownError(format!(
            "Library file at path {} does not exist",
            path.to_string_lossy()
        ))
    })?;

    Ok(path.to_string_lossy().to_string())
}

pub fn get_processor_arguments<T: AsRef<str>>(
    libraries_path: &Path,
    arguments: &[T],
    data: &HashMap<String, SidedDataEntry>,
) -> Result<Vec<String>> {
    let mut new_arguments = Vec::new();

    for argument in arguments {
        let trimmed_arg = &argument.as_ref()[1..argument.as_ref().len() - 1];
        if argument.as_ref().starts_with('{') {
            if let Some(entry) = data.get(trimmed_arg) {
                new_arguments.push(if entry.client.starts_with('[') {
                    get_lib_path(
                        libraries_path,
                        &entry.client[1..entry.client.len() - 1],
                        true,
                    )?
                } else {
                    entry.client.clone()
                })
            }
        } else if argument.as_ref().starts_with('[') {
            new_arguments.push(get_lib_path(libraries_path, trimmed_arg, true)?)
        } else {
            new_arguments.push(argument.as_ref().to_string())
        }
    }

    Ok(new_arguments)
}