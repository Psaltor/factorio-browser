use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const BASE_URL: &str = "https://multiplayer.factorio.com";

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub name: String,
    pub version: String,
}

/// Error type for API operations
#[derive(Debug)]
pub enum ApiError {
    RequestFailed(reqwest::Error),
    InvalidResponse(String),
    AuthenticationFailed,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::RequestFailed(e) => write!(f, "Request failed: {}", e),
            ApiError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            ApiError::AuthenticationFailed => write!(f, "Authentication failed"),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::RequestFailed(err)
    }
}

impl FactorioClient {
    /// Create a new Factorio API client
    pub fn new(username: String, token: String) -> Self {
        Self {
            client: Client::new(),
            username,
            token,
        }
    }

    /// Create a new client wrapped in Arc for sharing
    pub fn new_shared(username: String, token: String) -> Arc<Self> {
        Arc::new(Self::new(username, token))
    }

    /// Fetch all public game servers (requires authentication)
    pub async fn get_games(&self) -> Result<Vec<GameServer>, ApiError> {
        let url = format!(
            "{}/get-games?username={}&token={}",
            BASE_URL, self.username, self.token
        );

        let response = self.client.get(&url).send().await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ApiError::AuthenticationFailed);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::InvalidResponse(format!("{}: {}", status, body)));
        }

        Ok(response.json().await?)
    }

    /// Fetch detailed server info (no auth required)
    pub async fn get_game_details(&self, game_id: u64) -> Result<GameDetails, ApiError> {
        let url = format!("{}/get-game-details/{}", BASE_URL, game_id);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::InvalidResponse(format!("{}: {}", status, body)));
        }

        Ok(response.json().await?)
    }
}
