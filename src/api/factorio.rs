use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

const BASE_URL: &str = "https://multiplayer.factorio.com";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_RESPONSE_SIZE: usize = 16 * 1024 * 1024;

/// Game time that can be returned as either number (version 1.1+) or string (versions 0.16-1.0)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum GameTime {
    Number(u64),
    String(String),
}

impl GameTime {
    pub fn as_u64(&self) -> u64 {
        match self {
            GameTime::Number(n) => *n,
            GameTime::String(s) => s.parse().unwrap_or(0),
        }
    }
}

impl From<GameTime> for u64 {
    fn from(gt: GameTime) -> u64 {
        gt.as_u64()
    }
}

/// Factorio API client for the matchmaking API
#[derive(Clone)]
pub struct FactorioClient {
    client: Client,
    username: String,
    token: String,
}

/// Application version information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApplicationVersion {
    pub game_version: String,
    pub build_version: u32,
    pub build_mode: String,
    pub platform: String,
}

/// Server information from the get-games endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameServer {
    pub game_id: u64,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub max_players: u32,
    #[serde(default)]
    pub players: Vec<String>,
    pub game_time_elapsed: GameTime,
    pub has_password: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub mod_count: u32,
    #[serde(default)]
    pub host_address: Option<String>,
    pub application_version: ApplicationVersion,
    #[serde(default)]
    pub has_mods: bool,
    #[serde(default)]
    pub headless_server: bool,
    #[serde(default)]
    pub server_id: Option<String>,
}

/// Detailed server information from get-game-details endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameDetails {
    pub game_id: u64,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub max_players: u32,
    #[serde(default)]
    pub players: Vec<String>,
    pub game_time_elapsed: GameTime,
    pub has_password: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    pub application_version: ApplicationVersion,
    #[serde(default)]
    pub mods: Vec<ModInfo>,
    #[serde(default)]
    pub host_address: Option<String>,
    #[serde(default)]
    pub has_mods: bool,
    #[serde(default)]
    pub headless_server: bool,
}

/// Mod information for detailed server view
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModInfo {
    pub name: String,
    pub version: String,
}

/// Error type for API operations
#[derive(Debug)]
pub enum ApiError {
    RequestFailed(reqwest::Error),
    HttpStatus(reqwest::StatusCode),
    InvalidResponse(String),
    AuthenticationFailed,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::RequestFailed(e) => write!(f, "Request failed: {}", e),
            ApiError::HttpStatus(status) => write!(f, "Factorio API returned HTTP {status}"),
            ApiError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            ApiError::AuthenticationFailed => write!(f, "Authentication failed"),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::RequestFailed(err.without_url())
    }
}

impl FactorioClient {
    /// Create a new client wrapped in Arc for sharing
    pub fn new_shared(username: String, token: String) -> Arc<Self> {
        Arc::new(Self {
            client: Client::builder()
                .connect_timeout(CONNECT_TIMEOUT)
                .timeout(REQUEST_TIMEOUT)
                .build()
                .expect("valid HTTP client configuration"),
            username,
            token,
        })
    }

    /// Fetch all public game servers (requires authentication)
    pub async fn get_games(&self) -> Result<Vec<GameServer>, ApiError> {
        let response = self
            .client
            .get(format!("{BASE_URL}/get-games"))
            .query(&[("username", &self.username), ("token", &self.token)])
            .send()
            .await
            .map_err(ApiError::from)?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ApiError::AuthenticationFailed);
        }

        if !response.status().is_success() {
            return Err(ApiError::HttpStatus(response.status()));
        }

        read_bounded_json(response).await
    }

    /// Fetch detailed server info (no auth required)
    pub async fn get_game_details(&self, game_id: u64) -> Result<GameDetails, ApiError> {
        let url = format!("{}/get-game-details/{}", BASE_URL, game_id);
        let response = self.client.get(&url).send().await.map_err(ApiError::from)?;

        if !response.status().is_success() {
            return Err(ApiError::HttpStatus(response.status()));
        }

        read_bounded_json(response).await
    }
}

async fn read_bounded_json<T: DeserializeOwned>(
    mut response: reqwest::Response,
) -> Result<T, ApiError> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_SIZE as u64)
    {
        return Err(ApiError::InvalidResponse(
            "response body is too large".to_string(),
        ));
    }

    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(ApiError::from)? {
        if body.len().saturating_add(chunk.len()) > MAX_RESPONSE_SIZE {
            return Err(ApiError::InvalidResponse(
                "response body is too large".to_string(),
            ));
        }
        body.extend_from_slice(&chunk);
    }

    serde_json::from_slice(&body)
        .map_err(|error| ApiError::InvalidResponse(format!("invalid JSON: {error}")))
}

#[cfg(test)]
mod tests {
    use super::ApiError;

    #[tokio::test]
    async fn request_errors_do_not_expose_credentials_in_urls() {
        let error = reqwest::Client::new()
            .get("http://127.0.0.1:0/?token=super-secret-token")
            .send()
            .await
            .expect_err("port zero must reject the request");

        assert!(error.url().is_some());
        let message = ApiError::from(error).to_string();
        assert!(!message.contains("super-secret-token"));
        assert!(!message.contains("127.0.0.1"));
    }
}
