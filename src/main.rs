use factorio_browser::api::factorio::FactorioClient;
// TODO: Re-enable API routes later
// use factorio_browser::api::routes::{get_server, get_server_history, get_servers, health};
use factorio_browser::components::app::{App, AppProps};
use factorio_browser::components::server_details::ServerDetails;
use factorio_browser::db::queries::DbClient;
use factorio_browser::db::models::CachedServer;
use factorio_browser::utils::strip_all_tags;
use rocket::form::FromForm;
use rocket::fs::{relative, NamedFile};
use rocket::http::Header;
use rocket::response::content::RawHtml;
use rocket::response::{Responder, Response};
use rocket::Request;
use std::path::{Path, PathBuf};
use rocket::{get, routes, State};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use yew::ServerRenderer;

/// Application state
struct AppState {
    db: Arc<DbClient>,
    factorio_client: Arc<FactorioClient>,
    last_error: Arc<RwLock<Option<String>>>,
    // Add cached servers
    cached_servers: Arc<RwLock<Vec<CachedServer>>>,
}

/// Query parameters for the main page
#[derive(Debug, FromForm, Default)]
struct IndexFilters {
    search: Option<String>,
    version: Option<String>,
    has_players: Option<bool>,
    no_password: Option<bool>,
    is_dedicated: Option<bool>,
    tags: Option<String>, // Comma-separated list of tags for OR filtering
}

/// Wrap HTML content with the page shell, optionally with video background
fn html_shell_with_video(title: &str, content: String, with_video: bool) -> String {
    let video_url = "https://lambs.cafe/wp-content/uploads/2025/12/space-age.mp4";
    
    let video_element = if with_video {
        format!(r#"<video class="video-background" autoplay muted loop playsinline preload="auto">
        <source src="{}" type="video/mp4">
    </video>"#, video_url)
    } else {
        String::new()
    };
    
    let body_class = if with_video { " class=\"has-video\"" } else { "" };
    
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <meta name="description" content="Find and explore public Factorio multiplayer servers. Browse servers by version, tags, player count, and more.">
    <meta name="keywords" content="Factorio, multiplayer, servers, server browser, gaming, factory">
    <meta name="author" content="lambs.cafe">
    <meta name="theme-color" content="#0d0d0f">
    
    <!-- Open Graph / Facebook -->
    <meta property="og:type" content="website">
    <meta property="og:title" content="{title}">
    <meta property="og:description" content="Find and explore public Factorio multiplayer servers. Browse servers by version, tags, player count, and more.">
    <meta property="og:image" content="/static/favicon.svg">
    <meta property="og:site_name" content="Factorio Server Browser">
    
    <!-- Twitter -->
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="{title}">
    <meta name="twitter:description" content="Find and explore public Factorio multiplayer servers. Browse servers by version, tags, player count, and more.">
    <meta name="twitter:image" content="/static/favicon.svg">
    
    <link rel="icon" type="image/svg+xml" href="/static/favicon.svg">
    <link rel="stylesheet" href="/static/style.css">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600&family=Titillium+Web:wght@300;400;600;700&display=swap" rel="stylesheet">
</head>
<body{body_class}>
    {video}
    {content}
    <script src="/static/sort.js" defer></script>
</body>
</html>"##,
        title = title,
        body_class = body_class,
        video = video_element,
        content = content
    )
}

/// Main SSR route - renders the Yew app to HTML
#[get("/?<filters..>")]
async fn index(state: &State<Arc<AppState>>, filters: IndexFilters) -> RawHtml<String> {
    // Use cached servers instead of querying DB
    let servers = state.cached_servers.read().await.clone();
    let error = state.last_error.read().await.clone();

    let props = AppProps {
        servers,
        error,
        search: filters.search.unwrap_or_default(),
        version: filters.version.unwrap_or_default(),
        has_players: filters.has_players.unwrap_or(false),
        no_password: filters.no_password.unwrap_or(false),
        is_dedicated: filters.is_dedicated.unwrap_or(false),
        tags: filters.tags.unwrap_or_default(),
    };

    let renderer = ServerRenderer::<App>::with_props(move || props.clone());
    let html_content = renderer.render().await;

    RawHtml(html_shell_with_video("Factorio Server Browser", html_content, true))
}

/// Server details page
#[get("/server/<game_id>")]
async fn server_details_page(state: &State<Arc<AppState>>, game_id: u64) -> RawHtml<String> {
    use factorio_browser::components::server_details::ModEntry;
    
    // Get server from in-memory cache (avoids race condition during DB refresh)
    let server = state.cached_servers.read().await
        .iter()
        .find(|s| s.game_id == game_id)
        .cloned();
    
    // Fetch fresh details from API for players and mods
    let (players, mods) = match state.factorio_client.get_game_details(game_id).await {
        Ok(details) => (
            details.players,
            details.mods.into_iter().map(|m| ModEntry {
                name: m.name,
                version: m.version,
            }).collect(),
        ),
        Err(_) => (Vec::new(), Vec::new()),
    };
    
    // Fetch raw history and fill gaps with 0-player entries
    // Since we only record when players > 0, we need to fill in the timeline
    let raw_history = state
        .db
        .get_server_history(game_id, 24)
        .await
        .unwrap_or_default();
    
    let history = fill_history_gaps(raw_history);

    match server {
        Some(server) => {
            let title = format!("{} - Factorio Server Browser", strip_all_tags(&server.name));
            let props = factorio_browser::components::server_details::ServerDetailsProps { 
                server, 
                history,
                players,
                mods,
            };
            let renderer = ServerRenderer::<ServerDetails>::with_props(move || props.clone());
            let html_content = renderer.render().await;
            RawHtml(html_shell_with_video(&title, html_content, true))
        }
        None => {
            let html_content = r#"
                <div class="min-h-screen flex flex-col">
                    <header class="bg-bg-card/65 backdrop-blur-[10px] border-b border-border-subtle py-8 px-6">
                        <div class="max-w-[1400px] mx-auto text-center">
                            <h1 class="text-4xl font-bold text-text-bright">Server Not Found</h1>
                        </div>
                    </header>
                    <main class="flex-1 max-w-[1400px] mx-auto py-8 px-6 w-full">
                        <div class="text-center py-8 bg-status-full/10 border border-status-full/30 rounded-md text-status-full">
                            <p class="mb-4">
                                The requested server could not be found.<br/>
                                If you viewed this page previously, the server may have restarted and triggered a new game_id.<br/>
                                <b>It's a limitation of the Factorio Matchmaking API.</b>
                            </p>
                            <a href="/" class="text-accent-primary hover:text-accent-secondary transition-colors duration-200">
                                ← Back to Server List
                            </a>
                        </div>
                    </main>
                </div>
            "#
            .to_string();
            RawHtml(html_shell_with_video("Server Not Found", html_content, true))
        }
    }
}

/// Wrapper for NamedFile that adds caching headers
pub struct CachedFile(NamedFile);

impl<'r> Responder<'r, 'static> for CachedFile {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        Response::build_from(self.0.respond_to(req)?)
            // Cache for 1 day, revalidate with server
            .header(Header::new("Cache-Control", "public, max-age=86400, must-revalidate"))
            .ok()
    }
}

/// Serve static files from the static directory with caching headers
#[get("/static/<file..>")]
async fn static_files(file: PathBuf) -> Option<CachedFile> {
    let path = Path::new(relative!("static")).join(file);
    NamedFile::open(path).await.ok().map(CachedFile)
}

/// Fill gaps in history data with 0-player entries
/// Since we only record when players > 0, we need to fill in periods of inactivity
fn fill_history_gaps(raw_history: Vec<factorio_browser::db::models::ServerHistory>) -> Vec<factorio_browser::components::server_details::HistoryEntry> {
    use chrono::{DateTime, Duration, Utc};
    use factorio_browser::components::server_details::HistoryEntry;
    use std::collections::HashMap;
    
    let now = Utc::now();
    
    // Create a map of hour -> player counts for that hour
    let mut hourly_counts: HashMap<i64, Vec<usize>> = HashMap::new();
    
    for record in &raw_history {
        if let Ok(recorded_at) = DateTime::parse_from_rfc3339(&record.recorded_at) {
            // Calculate hours ago (0 = current hour, 23 = 23 hours ago)
            let hours_ago = (now - recorded_at.with_timezone(&Utc)).num_hours();
            if hours_ago >= 0 && hours_ago < 24 {
                hourly_counts
                    .entry(hours_ago)
                    .or_default()
                    .push(record.player_count);
            }
        }
    }
    
    // Generate 24 hourly entries (newest first to match expected order)
    // Each entry represents the average player count for that hour, or 0 if no data
    (0..24)
        .map(|hours_ago| {
            let avg_count = hourly_counts
                .get(&hours_ago)
                .map(|counts| counts.iter().sum::<usize>() / counts.len().max(1))
                .unwrap_or(0);
            
            let timestamp = now - Duration::hours(hours_ago);
            HistoryEntry {
                player_count: avg_count,
                recorded_at: timestamp.to_rfc3339(),
            }
        })
        .collect()
}

/// Sanitize error messages to remove sensitive information like URLs with credentials
fn sanitize_error(error: &str) -> String {
    // Remove URLs that might contain credentials
    if error.contains("http://") || error.contains("https://") {
        // Generic error message without exposing the URL
        if error.contains("get-games") || error.contains("multiplayer.factorio.com") {
            return "Failed to connect to Factorio API. Please try again later.".to_string();
        }
        return "A network error occurred. Please try again later.".to_string();
    }
    // For other errors, just return a generic message to be safe
    "An error occurred while fetching server data.".to_string()
}

/// Background task to periodically refresh server data
async fn refresh_servers(state: Arc<AppState>) {
    loop {
        println!("Refreshing server data...");

        match state.factorio_client.get_games().await {
            Ok(servers) => {
                let count = servers.len();

                // Record history before caching
                if let Err(e) = state.db.record_player_counts(&servers).await {
                    eprintln!("Failed to record history: {}", e);
                }

                // Cache the servers in DB
                match state.db.cache_servers(servers).await {
                    Ok(_) => {
                        println!("Cached {} servers", count);
                        *state.last_error.write().await = None;
                        
                        // Update in-memory cache from DB
                        if let Ok(all_servers) = state.db.get_all_servers().await {
                            *state.cached_servers.write().await = all_servers;
                        }
                    }
                    Err(e) => {
                        let raw_msg = format!("Failed to cache servers: {}", e);
                        eprintln!("{}", raw_msg);
                        // Display sanitized message to users
                        *state.last_error.write().await = Some("Failed to update server cache.".to_string());
                    }
                }

                // Clean up old history
                if let Err(e) = state.db.cleanup_old_history().await {
                    eprintln!("Failed to cleanup history: {}", e);
                }
            }
            Err(e) => {
                let raw_msg = format!("Failed to fetch servers: {}", e);
                eprintln!("{}", raw_msg);
                // Display sanitized message to users - never expose raw error with URLs/credentials
                *state.last_error.write().await = Some(sanitize_error(&raw_msg));
            }
        }

        // Wait before next refresh (60 seconds)
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Get configuration from environment variables
    let username = std::env::var("FACTORIO_USERNAME").unwrap_or_else(|_| {
        eprintln!("Warning: FACTORIO_USERNAME not set, API calls will fail");
        String::new()
    });

    let token = std::env::var("FACTORIO_TOKEN").unwrap_or_else(|_| {
        eprintln!("Warning: FACTORIO_TOKEN not set, API calls will fail");
        String::new()
    });

    let db_url = std::env::var("SURREAL_URL").unwrap_or_else(|_| "mem://".to_string());
    let db_ns = std::env::var("SURREAL_NS").unwrap_or_else(|_| "factorio".to_string());
    let db_name = std::env::var("SURREAL_DB").unwrap_or_else(|_| "browser".to_string());
    let db_user = std::env::var("SURREAL_USER").ok();
    let db_pass = std::env::var("SURREAL_PASS").ok();

    // Initialize database
    let db = DbClient::connect(
        &db_url,
        &db_ns,
        &db_name,
        db_user.as_deref(),
        db_pass.as_deref(),
    )
    .await
    .expect("Failed to connect to database");

    let db = Arc::new(db);

    // Initialize Factorio API client
    let factorio_client = FactorioClient::new_shared(username, token);

    // Create application state with empty cache
    let app_state = Arc::new(AppState {
        db: db.clone(),
        factorio_client: factorio_client.clone(),
        last_error: Arc::new(RwLock::new(None)),
        cached_servers: Arc::new(RwLock::new(Vec::new())),
    });

    // Start background refresh task
    let refresh_state = app_state.clone();
    tokio::spawn(async move {
        refresh_servers(refresh_state).await;
    });

    // Build and launch Rocket server
    rocket::build()
        .manage(app_state.db.clone())
        .manage(app_state)
        .mount("/", routes![index, server_details_page, static_files])
        // TODO: Re-enable API routes later
        // .mount("/", routes![health, get_servers, get_server, get_server_history])
        .launch()
        .await?;

    Ok(())
}
