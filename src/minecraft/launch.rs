use event_emitter_rs::EventEmitter;
use std::{
    any::type_name,
    collections::HashMap,
    env::consts::OS,
    path::{PathBuf, MAIN_SEPARATOR_STR},
    process::Stdio,
    sync::Arc,
};
use tokio::{
    fs::{metadata, set_permissions},
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::Mutex,
};

use crate::{
    auth::AuthMethod,
    emit,
    error::Error,
    json::version::meta::vanilla::{Arguments, Element, JavaVersion, Value, VersionMeta},
    minecraft::version::ParseRule,
    util::json::read_json,
};

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;

use super::loaders::Loader;
use super::CLASSPATH_SEPARATOR;

pub enum Memory {
    Megabyte(u64),
    Gigabyte(u16),
}

pub struct Config<T: Loader> {
    pub game_dir: PathBuf,
    pub version: String,
    pub authentication: AuthMethod,
    pub memory: Option<Memory>,
    pub version_name: Option<String>,
    pub loader: Option<T>,
    pub java_version: Option<String>,
    pub runtime_dir: Option<PathBuf>,
    pub custom_java_args: Vec<String>,
    pub custom_args: Vec<String>,
}

pub async fn launch<T: Loader>(
    config: &Config<T>,
    emitter: Option<Arc<Mutex<EventEmitter>>>,
) -> crate::Result<Child> {
    let version_name = config
        .version_name
        .clone()
        .unwrap_or(if config.loader.is_some() {
            let name = format!(
                "{}-{}",
                type_name::<T>().split("::").last().unwrap_or("Custom"),
                config.version
            );
            name
        } else {
            config.version.to_string()
        });

    let mut arguments = Vec::<String>::with_capacity(100);
    let meta: VersionMeta = read_json(
        &config
            .game_dir
            .join("versions")
            .join(&version_name)
            .join(format!("{}.json", &version_name)),
    )
    .await?;

    let meta_arguments = meta.arguments.unwrap_or(Arguments {
        game: meta
            .minecraft_arguments
            .unwrap_or_default()
            .split(" ")
            .map(|argument| Element::String(argument.to_string()))
            .collect::<Vec<Element>>(),
        jvm: vec![
            Element::String("-Djava.library.path=${natives_directory}".to_string()),
            Element::String("-cp".to_string()),
            Element::String("${classpath}".to_string()),
        ],
    });

    let mut variables = HashMap::<&'static str, String>::with_capacity(20);

    let mut insert_var = |key: &'static str, value: String| {
        variables.insert(key, value);
    };

    // Authentication variables
    match &config.authentication {
        AuthMethod::Microsoft {
            username,
            xuid,
            uuid,
            access_token,
            ..
        } => {
            insert_var("${auth_player_name}", username.clone());
            insert_var("${auth_xuid}", xuid.clone());
            insert_var("${auth_uuid}", uuid.clone());
            insert_var("${auth_access_token}", access_token.clone());
            insert_var("${user_type}", "msa".to_string());
        }
        AuthMethod::Offline { username, uuid } => {
            insert_var("${auth_player_name}", username.clone());
            insert_var("${auth_xuid}", uuid.clone());
            insert_var("${auth_uuid}", uuid.clone());
            insert_var("${auth_access_token}", "token".to_string());
            insert_var("${user_type}", "mojang".to_string());
        }
    }
    // Using original Minecraft launcher's client id for authentication.
    insert_var("${clientid}", "00000000402b5328".to_string());
    insert_var("${user_properties}", "".to_string());

    // Launcher variables
    insert_var("${launcher_name}", env!("CARGO_PKG_NAME").to_string());
    insert_var("${launcher_version}", env!("CARGO_PKG_VERSION").to_string());

    // Game configuration variables
    insert_var("${version_name}", version_name.clone());
    insert_var(
        "${game_directory}",
        config.game_dir.to_string_lossy().into_owned(),
    );
    insert_var(
        "${assets_root}",
        config
            .game_dir
            .join("assets")
            .to_string_lossy()
            .into_owned(),
    );
    insert_var(
        "${game_assets}",
        config
            .game_dir
            .join("assets")
            .join("virtual")
            .join("legacy")
            .to_string_lossy()
            .into_owned(),
    );
    insert_var("${assets_index_name}", meta.asset_index.id);
    insert_var("${version_type}", meta.r#type);
    insert_var(
        "${natives_directory}",
        config
            .game_dir
            .join("natives")
            .join(&config.version)
            .display()
            .to_string(),
    );
    insert_var("${classpath}", {
        let lib_path = config.game_dir.join("libraries");
        let mut cp = Vec::<String>::new();

        meta.libraries.iter().for_each(|lib| {
            let mut push_path_if_valid = |path: Option<String>| {
                if let Some(valid_path) = path {
                    let formatted_path = valid_path.replace("/", MAIN_SEPARATOR_STR);
                    if !formatted_path.is_empty() {
                        cp.push(lib_path.join(formatted_path).to_string_lossy().into_owned());
                    }
                }
            };

            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    if lib.rules.parse_rule() {
                        push_path_if_valid(artifact.path.clone());
                    }
                }

                if let Some(classifiers) = &downloads.classifiers {
                    let natives_path = match OS {
                        "windows" => &classifiers.natives_windows,
                        "linux" => &classifiers.natives_linux,
                        "macos" => &classifiers.natives_osx,
                        _ => panic!("Unknown operating system."),
                    };

                    if let Some(natives) = natives_path {
                        push_path_if_valid(natives.path.clone());
                    }
                }
            }
        });

        cp.push(
            config
                .game_dir
                .join("versions")
                .join(&version_name)
                .join(format!("{}.jar", &version_name))
                .to_string_lossy()
                .into_owned(),
        );

        cp.join(CLASSPATH_SEPARATOR)
    });

    fn replace_each(variables: &HashMap<&'static str, String>, arg: String) -> String {
        let mut arg = arg;
        for (k, v) in variables {
            if arg.contains(*k) {
                arg = arg.replace(*k, v);
            }
        }
        arg
    }

    match &config.memory {
        Some(memory) => arguments.push(format!(
            "-Xmx{}",
            match memory {
                Memory::Gigabyte(m) => format!("{}G", m),
                Memory::Megabyte(m) => format!("{}M", m),
            }
        )),
        None => arguments.push("-Xmx2G".to_string()),
    }

    meta_arguments.jvm.iter().for_each(|arg| match arg {
        Element::String(e) => arguments.push(replace_each(&variables, e.clone())),
        Element::Class(e) => {
            if e.rules.parse_rule() {
                match &e.value {
                    Value::Single(e) => arguments.push(replace_each(&variables, e.clone())),
                    Value::Multiple(e) => {
                        e.iter()
                            .for_each(|v| arguments.push(replace_each(&variables, v.clone())));
                    }
                }
            }
        }
    });

    arguments.push(meta.main_class.to_owned());

    meta_arguments.game.iter().for_each(|arg| {
        if let Element::String(e) = arg {
            arguments.push(replace_each(&variables, e.clone()))
        }
    });

    let runtime_dir = config
        .runtime_dir
        .clone()
        .unwrap_or(config.game_dir.join("runtime"))
        .join(
            meta.java_version
                .unwrap_or(JavaVersion {
                    component: "jre-legacy".to_string(),
                    major_version: 0,
                })
                .component,
        );

    #[cfg(not(target_os = "macos"))]
    let java_path = runtime_dir.join("bin").join("java");

    #[cfg(target_os = "macos")]
    let java_path = runtime_dir
        .join("jre.bundle")
        .join("Contents")
        .join("Home")
        .join("bin")
        .join("java");

    #[cfg(not(target_os = "windows"))]
    {
        let mut perms = metadata(&java_path).await?.permissions();
        perms.set_mode(0o755);
        set_permissions(&java_path, perms).await?;
    }

    println!("{:?}", arguments);

    let mut child = Command::new(java_path)
        .args(arguments)
        .stdout(Stdio::piped())
        .current_dir(&config.game_dir)
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or(Error::Take("Child -> stdout".to_string()))?;

    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Some(line) = reader.next_line().await.unwrap() {
            emit!(emitter, "console", line);
        }
    });

    Ok(child)
}
