
<br/>
<div align="center">

<h3 align="center">Lyceris</h3>
<p align="center">
An open source Minecraft launcher library written in Rust.
<br/>

![Static Badge](https://img.shields.io/badge/build-passing-brightgreen)
![Static Badge](https://img.shields.io/badge/crates.io-v0.2.0-orange)

</p>
</div>

## About The Project

![Product Screenshot](https://i.imgur.com/uQ13xHc.png)

Lyceris is written with functional programming paradigm to achieve simplicity. It supports Microsoft authentication, loaders like Fabric, Quilt (more will be implemented soon), multi-threaded control system and download parallelism. It also automatically downloads necessary Java version.
## Getting Started

- cargo
  ```sh
  cargo add lyceris
  ```

## Usage

This is the example implementation with using Quilt mod loader in version 1.20.
Don't forget to change the game directory path!
```rust
  async fn launch() {
    let config = Config {
        game_dir: "Path to the game directory".into(),
        version: "1.20".to_string(),
        authentication: auth::AuthMethod::Offline {
            username: "Notch".to_string(),
            uuid: "4c4ae28c-16c8-49f4-92a6-8d21e0d8b4a0".to_string(),
        },
        memory: None,
        java_version: None,
        version_name: None,
        loader: Some(Quilt("0.27.1".into())),
        runtime_dir: None,
        custom_args: vec![],
        custom_java_args: vec![],
    };

    let mut emitter = EventEmitter::new();

    // Handling single download progression
    emitter.on("single_download_progress", |(path, current, max): (String, u64, u64)| {
        println!("{}-{}/{}", path, current, max);
    });

    // Handling multi download progression
    emitter.on(
        "multiple_download_progress",
        |(path, current, max): (String, u64, u64)| {
            println!("{}-{}/{}", path, current, max);
        },
    );

    // Handling console outputs
    emitter.on("console", |line: String| {
        println!("Line: {}", line);
    });

    let emitter = Arc::new(Mutex::new(emitter));

    install(&config, Some(&emitter)).await?;

    let child = launch(&config, Some(emitter)).await?;

    child.wait().await?;

  }
```
## Roadmap

- [ ] Microsoft Authentication
- [ ] Forge mod loader support

See the [open issues](https://github.com/cubidron/lyceris/issues) for a full list of proposed features (and known issues).
## License

Distributed under the MIT License. See [MIT License](https://opensource.org/licenses/MIT) for more information.
