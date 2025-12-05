use crate::api::factorio::GameServer;
use crate::db::models::{CachedServer, NewCachedServer, NewServerHistory, ServerHistory};
use surrealdb::engine::any::{connect, Any};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

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
        let db = connect(url)
            .await
            .map_err(|e| DbError::Connection(e.to_string()))?;

        // Sign in if credentials are provided (required for remote connections)
        if url.starts_with("ws://") || url.starts_with("wss://") {
            let user = username.unwrap_or("root");
            let pass = password.unwrap_or("root");
            db.signin(Root {
                username: user,
                password: pass,
            })
            .await
            .map_err(|e| DbError::Connection(e.to_string()))?;
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
        
        // Begin transaction for atomic delete + insert
        self.db.query("BEGIN TRANSACTION").await?;
        
        // Delete all existing servers
        if let Err(e) = self.db.query("DELETE FROM servers").await {
            self.db.query("CANCEL TRANSACTION").await.ok();
            return Err(e.into());
        }
        
        // Insert in batches for better performance
        const BATCH_SIZE: usize = 500;
        for chunk in new_servers.chunks(BATCH_SIZE) {
            if let Err(e) = self.db
                .insert::<Vec<CachedServer>>("servers")
                .content(chunk.to_vec())
                .await
            {
                self.db.query("CANCEL TRANSACTION").await.ok();
                return Err(e.into());
            }
        }
        
        // Commit transaction
        self.db.query("COMMIT TRANSACTION").await?;

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 500 {
            eprintln!("[DB SLOW] cache_servers took {:?} for {} servers", elapsed, count);
        }

        Ok(count)
    }

    /// Record player count for history tracking (batch operation)
    pub async fn record_player_counts(&self, servers: &[GameServer]) -> Result<(), DbError> {
        let start = std::time::Instant::now();
        let now = chrono::Utc::now().to_rfc3339();

        // Only record history for servers with players (significant data reduction)
        let history_records: Vec<NewServerHistory> = servers
            .iter()
            .filter(|server| !server.players.is_empty())
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
        let _: Vec<ServerHistory> = self.db
            .insert("server_history")
            .content(history_records)
            .await?;

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 500 {
            eprintln!("[DB SLOW] record_player_counts took {:?} for {} records", elapsed, record_count);
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
        let history: Vec<ServerHistory> = self
            .db
            .query(
                r#"
                SELECT * FROM server_history 
                WHERE game_id = $game_id 
                ORDER BY recorded_at DESC 
                LIMIT $limit
                "#,
            )
            .bind(("game_id", game_id))
            .bind(("limit", hours * 60)) // Assuming ~1 record per minute
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

    /// Get the timestamp of the last cache update
    pub async fn get_last_cache_time(&self) -> Result<Option<String>, DbError> {
        let result: Vec<CachedServer> = self
            .db
            .query("SELECT cached_at FROM servers LIMIT 1")
            .await?
            .take(0)?;

        Ok(result.first().map(|s| s.cached_at.clone()))
    }
}

