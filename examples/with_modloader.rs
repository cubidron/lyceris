use std::env;

use lyceris::minecraft::{
    config::ConfigBuilder, install::install, launch::launch, loader::fabric::Fabric,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let config = ConfigBuilder::new(
        current_dir.join("game"),
        "1.21.4".to_string(),
        lyceris::auth::AuthMethod::Offline {
            username: "Lyceris".to_string(),
            // If none given, it will be generated.
            uuid: None,
        },
    )
    // You can use Fabric, Quilt or Forge here.
    .loader(Fabric("0.16.9"))
    .build();

    // Install method also checks for broken files
    // and downloads them again if they are broken.
    install(&config, None).await?;

    // This method never downloads any file and just runs the game.
    launch(&config, None).await?.wait().await?;

    Ok(())
}
