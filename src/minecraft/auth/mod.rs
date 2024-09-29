use serde::Deserialize;

#[derive(Deserialize)]
pub enum AuthMethod {
    Offline(String),
    Online(String),
}

// const AUTHENTICATION_URL: &str = "https://login.live.com/oauth20_authorize.srf?client_id={}&response_type=code&redirect_uri={}&scope=XboxLive.signin%20offline_access";
// const CLIENT_ID: &str = "4b9a3a73-2f8c-477c-a9f0-03e9a537ae2b";
// impl Online {
//     pub async fn authenticate() -> Result<crate::error::Error> {
//         let url = format!(AUTHENTICATION_URL, CLIENT_ID);
//     }
// }
