pub enum AuthMethod{
    Offline(Offline),
    Online(Online)
}

pub struct Offline{
    pub username : String
}

pub struct Online{}