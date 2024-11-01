use online::Online;
use serde::Deserialize;

pub mod online;

#[derive(Deserialize)]
pub enum AuthMethod {
    Offline(String),
    Online(Online),
}
