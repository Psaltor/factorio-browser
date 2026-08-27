use crate::api::factorio::GameServer;
use crate::db::models::{CachedServer, NewCachedServer, NewServerHistory, ServerHistory};
use surrealdb::Surreal;
use surrealdb::engine::any::{Any, connect};
use surrealdb::opt::auth::Root;

/// Database client wrapper for SurrealDB operations
#[derive(Clone)]
pub struct DbClient {
    db: Surreal<Any>,
}

/// Error type for database operations
#[derive(Debug)]
pub enum DbError {
    Connection(String),
    Query(String),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Connection(msg) => write!(f, "Connection error: {}", msg),
            DbError::Query(msg) => write!(f, "Query error: {}", msg),
        }
    }
}

impl std::error::Error for DbError {}

impl From<surrealdb::Error> for DbError {
    fn from(err: surrealdb::Error) -> Self {
        DbError::Query(err.to_string())
    }
}

impl DbClient {
    /// Connect to SurrealDB and initialize the database
    pub async fn connect(
        url: &str,
        namespace: &str,
        database: &str,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self, DbError> {
        let is_remote = ["ws://", "wss://", "http://", "https://"]
            .iter()
            .any(|scheme| url.starts_with(scheme));
        if is_remote && matches!((username, password), (Some(_), None) | (None, Some(_))) {
            return Err(DbError::Connection(
                "SURREAL_USER and SURREAL_PASS must be provided together".to_string(),
            ));
        }

        let db = connect(url)
            .await
            .map_err(|e| DbError::Connection(e.to_string()))?;

        if is_remote {
            match (username, password) {
                (Some(user), Some(pass)) => {
                    db.signin(Root {
                        username: user,
                        password: pass,
                    })
                    .await
                    .map_err(|e| DbError::Connection(e.to_string()))?;
                }
                (None, None) => {}
                _ => {
                    return Err(DbError::Connection(
                        "SURREAL_USER and SURREAL_PASS must be provided together".to_string(),
                    ));
                }
            }
        }

        db.use_ns(namespace)
            .use_db(database)
            .await
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let client = Self { db };
        client.init_schema().await?;

        Ok(client)
    }

    /// Initialize database schema
    async fn init_schema(&self) -> Result<(), DbError> {
        // Create servers table with unique game_id index
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS servers SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS game_id ON servers TYPE int;
                DEFINE FIELD IF NOT EXISTS name ON servers TYPE string;
                DEFINE FIELD IF NOT EXISTS description ON servers TYPE string;
                DEFINE FIELD IF NOT EXISTS max_players ON servers TYPE int;
                DEFINE FIELD IF NOT EXISTS player_count ON servers TYPE int;
                DEFINE FIELD IF NOT EXISTS players ON servers TYPE array<string>;
                DEFINE FIELD IF NOT EXISTS game_time_elapsed ON servers TYPE int;
                DEFINE FIELD IF NOT EXISTS has_password ON servers TYPE bool;
                DEFINE FIELD IF NOT EXISTS tags ON servers TYPE array<string>;
                DEFINE FIELD IF NOT EXISTS mod_count ON servers TYPE int;
                DEFINE FIELD IF NOT EXISTS game_version ON servers TYPE string;
                DEFINE FIELD IF NOT EXISTS build_version ON servers TYPE int;
                DEFINE FIELD IF NOT EXISTS host_address ON servers TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS headless_server ON servers TYPE bool;
                DEFINE FIELD IF NOT EXISTS cached_at ON servers TYPE string;
                DEFINE INDEX IF NOT EXISTS game_id_idx ON servers FIELDS game_id UNIQUE;
                "#,
            )
            .await?;

        // Create server_history table
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS server_history SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS game_id ON server_history TYPE int;
                DEFINE FIELD IF NOT EXISTS player_count ON server_history TYPE int;
                DEFINE FIELD IF NOT EXISTS recorded_at ON server_history TYPE string;
                DEFINE INDEX IF NOT EXISTS history_game_idx ON server_history FIELDS game_id;
                DEFINE INDEX IF NOT EXISTS history_time_idx ON server_history FIELDS recorded_at;
                DEFINE INDEX IF NOT EXISTS history_game_time_idx ON server_history FIELDS game_id, recorded_at;
                "#,
            )
            .await?;

        Ok(())
    }

    /// Cache a list of servers from the API (batch operation)
    /// Uses a transaction to ensure atomicity - either all servers are updated or none are
    pub async fn cache_servers(&self, servers: Vec<GameServer>) -> Result<usize, DbError> {
        let start = std::time::Instant::now();
        let count = servers.len();

        // Use native insert_many for better performance
        let new_servers: Vec<NewCachedServer> = servers.into_iter().map(|s| s.into()).collect();

        self.db
            .query(
                r#"
                BEGIN TRANSACTION;
                DELETE FROM servers;
                INSERT INTO servers $servers;
                COMMIT TRANSACTION;
                "#,
            )
            .bind(("servers", new_servers))
            .await?
            .check()?;

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 500 {
            eprintln!(
                "[DB SLOW] cache_servers took {:?} for {} servers",
                elapsed, count
            );
        }

        Ok(count)
    }

    /// Record player count for history tracking (batch operation)
    pub async fn record_player_counts(&self, servers: &[GameServer]) -> Result<(), DbError> {
        let start = std::time::Instant::now();
        let now = chrono::Utc::now().to_rfc3339();

        let history_records: Vec<NewServerHistory> = servers
            .iter()
            .map(|server| NewServerHistory {
                game_id: server.game_id,
                player_count: server.players.len(),
                recorded_at: now.clone(),
            })
            .collect();

        if history_records.is_empty() {
            return Ok(());
        }

        let record_count = history_records.len();

        // Use native insert for better performance
        let _: Vec<ServerHistory> = self
            .db
            .insert("server_history")
            .content(history_records)
            .await?;

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 500 {
            eprintln!(
                "[DB SLOW] record_player_counts took {:?} for {} records",
                elapsed, record_count
            );
        }

        Ok(())
    }

    /// Get all cached servers
    pub async fn get_all_servers(&self) -> Result<Vec<CachedServer>, DbError> {
        let servers: Vec<CachedServer> = self
            .db
            .query("SELECT * FROM servers ORDER BY player_count DESC")
            .await?
            .take(0)?;

        Ok(servers)
    }

    /// Get a specific server by game_id
    pub async fn get_server(&self, game_id: u64) -> Result<Option<CachedServer>, DbError> {
        let mut result: Vec<CachedServer> = self
            .db
            .query("SELECT * FROM servers WHERE game_id = $game_id")
            .bind(("game_id", game_id))
            .await?
            .take(0)?;

        Ok(result.pop())
    }

    /// Get player count history for a server
    pub async fn get_server_history(
        &self,
        game_id: u64,
        hours: u32,
    ) -> Result<Vec<ServerHistory>, DbError> {
        let hours = hours.clamp(1, 168);
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(i64::from(hours));
        let history: Vec<ServerHistory> = self
            .db
            .query(
                r#"
                SELECT * FROM server_history 
                WHERE game_id = $game_id AND recorded_at >= $cutoff
                ORDER BY recorded_at DESC 
                "#,
            )
            .bind(("game_id", game_id))
            .bind(("cutoff", cutoff.to_rfc3339()))
            .await?
            .take(0)?;

        Ok(history)
    }

    /// Clean up old history records (keep last 24 hours)
    pub async fn cleanup_old_history(&self) -> Result<(), DbError> {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);

        self.db
            .query("DELETE FROM server_history WHERE recorded_at < $cutoff")
            .bind(("cutoff", cutoff.to_rfc3339()))
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::DbClient;
    use crate::api::factorio::{ApplicationVersion, GameServer, GameTime};

    fn server(game_id: u64, name: &str, players: &[&str]) -> GameServer {
        GameServer {
            game_id,
            name: name.to_string(),
            description: String::new(),
            max_players: 20,
            players: players.iter().map(|player| player.to_string()).collect(),
            game_time_elapsed: GameTime::Number(60),
            has_password: false,
            tags: Vec::new(),
            mod_count: 0,
            host_address: None,
            application_version: ApplicationVersion {
                game_version: "2.0.0".to_string(),
                build_version: 1,
                build_mode: "headless".to_string(),
                platform: "linux64".to_string(),
            },
            has_mods: false,
            headless_server: true,
            server_id: None,
        }
    }

    #[tokio::test]
    async fn cache_replacement_rolls_back_on_duplicate_game_ids() {
        let db = DbClient::connect("mem://", "test", "cache_rollback", None, None)
            .await
            .unwrap();
        db.cache_servers(vec![server(1, "original", &[])])
            .await
            .unwrap();

        let result = db
            .cache_servers(vec![
                server(2, "duplicate one", &[]),
                server(2, "duplicate two", &[]),
            ])
            .await;
        assert!(result.is_err());

        let cached = db.get_all_servers().await.unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].game_id, 1);
        assert_eq!(cached[0].name, "original");
    }

    #[tokio::test]
    async fn empty_cache_replacement_clears_existing_servers() {
        let db = DbClient::connect("mem://", "test", "cache_clear", None, None)
            .await
            .unwrap();
        db.cache_servers(vec![server(1, "original", &[])])
            .await
            .unwrap();

        db.cache_servers(Vec::new()).await.unwrap();
        assert!(db.get_all_servers().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn records_zero_player_observations() {
        let db = DbClient::connect("mem://", "test", "zero_history", None, None)
            .await
            .unwrap();
        db.record_player_counts(&[server(1, "empty", &[])])
            .await
            .unwrap();

        let history = db.get_server_history(1, 24).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].player_count, 0);
    }

    #[tokio::test]
    async fn rejects_partial_remote_credentials_before_connecting() {
        let error = DbClient::connect("ws://127.0.0.1:1", "test", "test", Some("user"), None)
            .await
            .err()
            .unwrap();
        assert!(error.to_string().contains("must be provided together"));
    }
}
