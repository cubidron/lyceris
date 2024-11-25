use std::{
    env,
    process::{Child, Command},
};

use crate::{network::get, prelude::Result};

pub async fn start_server(
    api_uri: String,
    server_address: String,
    server_port: u64,
    token: String,
    local_ip: String,
    local_port: u64,
    proxy_name: String,
) -> Result<(Child, String)> {
    let port = get(format!("{}/request-port", api_uri))
        .await?
        .text()
        .await?;

    let current_dir = env::current_dir()?;

    println!("{}", current_dir.display());

    let exe_path = current_dir
        .join("src")
        .join("frpc")
        .join("bin")
        .join("frpc.exe");

    let cmd = Command::new(exe_path)
        .args([
            "tcp",
            format!("--server-addr={}", server_address).as_str(),
            format!("--server-port={}", server_port).as_str(),
            format!("--token={}", token).as_str(),
            format!("--local-ip={}", local_ip).as_str(),
            format!("--local-port={}", local_port).as_str(),
            format!("--remote-port={}", port).as_str(),
            format!("--proxy-name={}", proxy_name).as_str(),
        ])
        .spawn()?;

    Ok((cmd, format!("{}:{}", server_address, port)))
}
