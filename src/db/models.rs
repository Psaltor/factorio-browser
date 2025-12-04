use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

/// Cached server record stored in SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CachedServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub game_id: u64,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub max_players: u32,
    pub player_count: usize,
    #[serde(default)]
    pub players: Vec<String>,
    pub game_time_elapsed: u64,
    pub has_password: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub mod_count: u32,
    pub game_version: String,
    pub build_version: u32,
    #[serde(default)]
    pub host_address: Option<String>,
    pub cached_at: String,
}

/// Server history record for tracking player counts over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHistory {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub game_id: u64,
    pub player_count: usize,
    pub recorded_at: String,
}

/// Input type for creating a new cached server (without id)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCachedServer {
    pub game_id: u64,
    pub name: String,
    pub description: String,
    pub max_players: u32,
    pub player_count: usize,
    pub players: Vec<String>,
    pub game_time_elapsed: u64,
    pub has_password: bool,
    pub tags: Vec<String>,
    pub mod_count: u32,
    pub game_version: String,
    pub build_version: u32,
    pub host_address: Option<String>,
    pub cached_at: String,
}

/// Input type for creating a new history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewServerHistory {
    pub game_id: u64,
    pub player_count: usize,
    pub recorded_at: String,
}

impl From<crate::api::factorio::GameServer> for NewCachedServer {
    fn from(server: crate::api::factorio::GameServer) -> Self {
        Self {
            game_id: server.game_id,
            name: server.name,
            description: server.description,
            max_players: server.max_players,
            player_count: server.players.len(),
            players: server.players,
            game_time_elapsed: server.game_time_elapsed.as_u64(),
            has_password: server.has_password,
            tags: server.tags,
            mod_count: server.mod_count,
            game_version: server.application_version.game_version,
            build_version: server.application_version.build_version,
            host_address: server.host_address,
            cached_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

