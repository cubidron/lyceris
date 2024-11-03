use std::time::{SystemTime, UNIX_EPOCH};

use oauth2::{AuthUrl, ClientId, CsrfToken, RedirectUrl, Scope, TokenUrl};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use error::Error;

use crate::{error, network, prelude::Result, utils::decode_base64_url};

pub static CLIENT_ID: &str = "00000000402b5328";
pub static REDIRECT_URI: &str = "https://login.live.com/oauth20_desktop.srf";
pub static AUTH_URL: &str = "https://login.live.com/oauth20_authorize.srf";
pub static TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";

#[derive(Serialize, Deserialize, Debug)]
struct MSToken {
    access_token: String,
    refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct XboxToken {
    #[serde(rename = "IssueInstant")]
    issue_instant: String,
    #[serde(rename = "NotAfter")]
    not_after: String,
    #[serde(rename = "Token")]
    token: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftResponse {
    /// UUID of the Xbox account.
    /// Please note that this is not the Minecraft player's UUID
    pub username: String,
    /// The minecraft JWT access token
    pub access_token: String,
    /// How many seconds until the token expires
    pub expires_in: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct XstsToken {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: DisplayClaims,
}

#[derive(Serialize, Deserialize, Debug)]
struct DisplayClaims {
    xui: Vec<Xui>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Xui {
    uhs: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Skin {
    id: String,
    state: String,
    url: String,
    variant: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Cape {
    id: String,
    state: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserProfile {
    pub id: Option<String>,
    pub name: Option<String>,
    skins: Option<Vec<Skin>>,
    capes: Option<Vec<Cape>>,
    path: Option<String>,
    error: Option<String>,
    #[serde(rename = "errorMessage")]
    error_message: Option<String>
}

#[derive(Debug, Deserialize)]
pub struct MCJWTDecoded {
    xuid: String,
    agg: String,
    sub: String,
    auth: String,
    ns: String,
    roles: Vec<String>,
    iss: String,
    flags: Vec<String>,
    profiles: Profiles,
    platform: String,
    yuid: String,
    nbf: u64,
    exp: u64,
    iat: u64,
}

#[derive(Debug, Deserialize)]
struct Profiles {
    mc: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Online {
    pub xuid: String,
    pub exp: u64,
    pub uuid: String,
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    pub client_id: String,
}

impl Online {
    pub fn create_link() -> Result<String> {
        let auth_url = AuthUrl::new(AUTH_URL.to_string())?;
        let token_url = TokenUrl::new(TOKEN_URL.to_string())?;

        let client = oauth2::basic::BasicClient::new(
            ClientId::new(CLIENT_ID.to_string()),
            None,
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(RedirectUrl::new(REDIRECT_URI.to_string())?);

        let (authorize_url, _) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(
                "service::user.auth.xboxlive.com::MBI_SSL".to_string(),
            ))
            .url();

        Ok(authorize_url.to_string())
    }

    pub async fn authenticate(code: String) -> Result<Online> {
        let ms_token = Self::get_ms_token(&code).await?;
        let xbox_token = Self::get_xbox_token(&ms_token.access_token).await?;
        let xsts_token = Self::get_xsts_token(&xbox_token.token).await?;
        let userhash = xsts_token
            .display_claims
            .xui
            .get(0)
            .ok_or("No XUI claims found")
            .unwrap()
            .uhs
            .clone();
        let token = Self::get_minecraft_token(&xsts_token.token, &userhash).await?;
        let profile = Self::get_profile(token.access_token.clone()).await?;
        let jwt = Self::parse_login_token(&token.access_token)?;

        Ok(Self {
            xuid: jwt.xuid,
            exp: jwt.exp,
            uuid: profile.id.unwrap_or_default(),
            username: profile.name.unwrap_or_default(),
            access_token: token.access_token,
            refresh_token: ms_token.refresh_token,
            client_id: CLIENT_ID.to_string(),
        })
    }

    pub async fn validate(&self) -> bool {
        return self.exp < SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System time error").unwrap()
        .as_secs() as u64
    }

    pub async fn refresh(&self) -> Result<Online> {
        let token_response = Client::new()
            .post(TOKEN_URL)
            .form(&[
                ("client_id", CLIENT_ID),
                ("scope", "service::user.auth.xboxlive.com::MBI_SSL"),
                ("grant_type", "refresh_token"),
                ("redirect_uri", REDIRECT_URI),
                ("refresh_token", &self.refresh_token)
            ])
            .send()
            .await?;

        let ms_token: MSToken = token_response.json().await?;
        let xbox_token = Self::get_xbox_token(&ms_token.access_token).await?;
        let xsts_token = Self::get_xsts_token(&xbox_token.token).await?;
        let userhash = xsts_token
            .display_claims
            .xui
            .get(0)
            .ok_or("No XUI claims found")
            .unwrap()
            .uhs
            .clone();
        let token = Self::get_minecraft_token(&xsts_token.token, &userhash).await?;
        let profile = Self::get_profile(token.access_token.clone()).await?;
        let jwt = Self::parse_login_token(&token.access_token)?;

        Ok(Self {
            xuid: jwt.xuid,
            exp: jwt.exp,
            uuid: profile.id.unwrap_or_default(),
            username: profile.name.unwrap_or_default(),
            access_token: token.access_token,
            refresh_token: ms_token.refresh_token,
            client_id: CLIENT_ID.to_string(),
        })
    }

    async fn get_ms_token(code: &str) -> Result<MSToken> {
        let token_response = Client::new()
            .post(TOKEN_URL)
            .form(&[
                ("client_id", CLIENT_ID),
                ("scope", "service::user.auth.xboxlive.com::MBI_SSL"),
                ("code", code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", REDIRECT_URI),
            ])
            .send()
            .await?;

        let ms_token: MSToken = token_response.json().await?;
        Ok(ms_token)
    }

    /// Microsoft token'ını kullanarak Xbox token almak
    async fn get_xbox_token(ms_token: &str) -> Result<XboxToken> {
        let body = serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": ms_token
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        });

        let xbox_response =
            network::post("https://user.auth.xboxlive.com/user/authenticate", body).await?;
        let xbox_token: XboxToken = xbox_response.json().await?;
        Ok(xbox_token)
    }

    async fn get_xsts_token(xbox_token: &str) -> Result<XstsToken> {
        let body = serde_json::json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbox_token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        });

        let xsts_response =
            network::post("https://xsts.auth.xboxlive.com/xsts/authorize", body).await?;
        let xsts_token: XstsToken = xsts_response.json().await?;
        Ok(xsts_token)
    }

    async fn get_minecraft_token(xsts_token: &str, userhash: &str) -> Result<MinecraftResponse> {
        let body = serde_json::json!({
            "identityToken": format!("XBL3.0 x={};{}", userhash, xsts_token)
        });

        let minecraft_response = network::post(
            "https://api.minecraftservices.com/authentication/login_with_xbox",
            body,
        )
        .await?;
        let minecraft_token: MinecraftResponse = minecraft_response.json().await?;
        Ok(minecraft_token)
    }

    fn parse_login_token(mc_token: &str) -> Result<MCJWTDecoded> {
        let base64_url = mc_token
            .split('.')
            .nth(1)
            .ok_or(error::Error::UnknownError("Couldn't split".to_string()))?;
        let decoded_bytes = decode_base64_url(base64_url)?;
        let json_payload = String::from_utf8(decoded_bytes)?;

        let decoded: MCJWTDecoded = serde_json::from_str(&json_payload)?;

        Ok(decoded)
    }

    async fn get_profile(access_token: String) -> Result<UserProfile> {
        let api_url = "https://api.minecraftservices.com/minecraft/profile";
        let client = Client::new();

        let response = client
            .get(api_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        let profile = response.json::<UserProfile>().await?;

        if let Some(error) = profile.error {
            match error.as_str() {
                "NOT_FOUND" => return Err(Error::AuthenticationError("Could not find minecraft profile".to_string())),
                _ => return Err(Error::AuthenticationError(error))
            };
        } else {
            return Ok(profile)
        }
    }
}
