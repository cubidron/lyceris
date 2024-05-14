pub enum AuthMethod{
    Offline(String),
    Online(Online)
}

pub struct Online{}