use serde::Deserialize;

#[derive(Deserialize)]
pub enum AuthMethod{
    Offline(String),
    Online(Online)
}

#[derive(Deserialize)]
pub struct Online{}