use crate::db::models::CachedServer;
use crate::db::queries::DbClient;
use rocket::form::FromForm;
use rocket::serde::json::Json;
use rocket::{get, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Query parameters for server filtering
#[derive(Debug, FromForm, Default)]
pub struct ServerFilters {
    /// Search by server name
    pub search: Option<String>,
    /// Filter by game version
    pub version: Option<String>,
    /// Only show servers with players
    pub has_players: Option<bool>,
    /// Only show servers without password
    pub no_password: Option<bool>,
    /// Filter by mod count (minimum)
    pub min_mods: Option<u32>,
    /// Maximum number of results
    pub limit: Option<usize>,
}

/// API response for server list
#[derive(Debug, Serialize)]
pub struct ServersResponse {
    pub servers: Vec<CachedServer>,
    pub total: usize,
    pub cached_at: Option<String>,
}

/// API response for server details
#[derive(Debug, Serialize)]
pub struct ServerDetailsResponse {
    pub server: Option<CachedServer>,
    pub history: Vec<PlayerCountHistory>,
}

/// Player count history entry
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerCountHistory {
    pub player_count: usize,
    pub recorded_at: String,
}

/// Health check endpoint
#[get("/health")]
pub fn health() -> &'static str {
    "OK"
}

/// Get list of cached servers with optional filtering
#[get("/api/servers?<filters..>")]
pub async fn get_servers(
    db: &State<Arc<DbClient>>,
    filters: ServerFilters,
) -> Json<ServersResponse> {
    let all_servers = db.get_all_servers().await.unwrap_or_default();

    let filtered: Vec<CachedServer> = all_servers
        .into_iter()
        .filter(|s| {
            // Search filter
            if let Some(ref search) = filters.search {
                let search_lower = search.to_lowercase();
                if !s.name.to_lowercase().contains(&search_lower)
                    && !s.description.to_lowercase().contains(&search_lower)
                {
                    return false;
                }
            }

            // Version filter
            if let Some(ref version) = filters.version {
                if !s.game_version.starts_with(version) {
                    return false;
                }
            }

            // Has players filter
            if let Some(has_players) = filters.has_players {
                if has_players && s.player_count == 0 {
                    return false;
                }
            }

            // No password filter
            if let Some(no_password) = filters.no_password {
                if no_password && s.has_password {
                    return false;
                }
            }

            // Min mods filter
            if let Some(min_mods) = filters.min_mods {
                if s.mod_count < min_mods {
                    return false;
                }
            }

            true
        })
        .collect();

    let total = filtered.len();
    let servers = if let Some(limit) = filters.limit {
        filtered.into_iter().take(limit).collect()
    } else {
        filtered
    };

    let cached_at = servers.first().map(|s| s.cached_at.clone());

    Json(ServersResponse {
        servers,
        total,
        cached_at,
    })
}

/// Get details for a specific server by game_id
#[get("/api/servers/<game_id>")]
pub async fn get_server(db: &State<Arc<DbClient>>, game_id: u64) -> Json<ServerDetailsResponse> {
    let server = db.get_server(game_id).await.ok().flatten();
    let history = db
        .get_server_history(game_id, 24)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|h| PlayerCountHistory {
            player_count: h.player_count,
            recorded_at: h.recorded_at,
        })
        .collect();

    Json(ServerDetailsResponse { server, history })
}

/// Get player count history for a server
#[get("/api/servers/<game_id>/history?<hours>")]
pub async fn get_server_history(
    db: &State<Arc<DbClient>>,
    game_id: u64,
    hours: Option<u32>,
) -> Json<Vec<PlayerCountHistory>> {
    let limit = hours.unwrap_or(24);
    let history = db
        .get_server_history(game_id, limit)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|h| PlayerCountHistory {
            player_count: h.player_count,
            recorded_at: h.recorded_at,
        })
        .collect();

    Json(history)
}

