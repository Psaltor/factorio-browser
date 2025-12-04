use factory_tracker::api::factorio::FactorioClient;
use factory_tracker::api::routes::{get_server, get_server_history, get_servers, health};
use factory_tracker::components::app::{App, AppProps};
use factory_tracker::components::server_details::ServerDetails;
use factory_tracker::db::queries::DbClient;
use factory_tracker::db::models::CachedServer;
use factory_tracker::utils::strip_all_tags;
use rocket::form::FromForm;
use rocket::fs::FileServer;
use rocket::response::content::RawHtml;
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
    tags: Option<String>, // Comma-separated list of tags for OR filtering
}

/// Wrap HTML content with the page shell, optionally with video background
fn html_shell_with_video(title: &str, content: String, with_video: bool) -> String {
    let video_element = if with_video {
        r#"<video class="video-background" autoplay muted loop playsinline>
        <source src="https://lambs.cafe/wp-content/uploads/2025/12/space-age.mp4" type="video/mp4">
    </video>"#
    } else {
        ""
    };
    
    let body_class = if with_video { " class=\"has-video\"" } else { "" };
    
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
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
</html>"#,
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
        tags: filters.tags.unwrap_or_default(),
    };

    let renderer = ServerRenderer::<App>::with_props(move || props.clone());
    let html_content = renderer.render().await;

    RawHtml(html_shell_with_video("Factorio Server Browser", html_content, true))
}

/// Server details page
#[get("/server/<game_id>")]
async fn server_details_page(state: &State<Arc<AppState>>, game_id: u64) -> RawHtml<String> {
    use factory_tracker::components::server_details::{HistoryEntry, ModEntry};
    
    // Get cached server data
    let server = state.db.get_server(game_id).await.ok().flatten();
    
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
    
    let history = state
        .db
        .get_server_history(game_id, 24)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|h| HistoryEntry {
            player_count: h.player_count,
            recorded_at: h.recorded_at,
        })
        .collect();

    match server {
        Some(server) => {
            let title = format!("{} - Factorio Server Browser", strip_all_tags(&server.name));
            let props = factory_tracker::components::server_details::ServerDetailsProps { 
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
                            <p class="mb-4">The requested server could not be found.</p>
                            <a href="/" class="text-accent-primary hover:text-accent-secondary transition-colors duration-200">← Back to Server List</a>
                        </div>
                    </main>
                </div>
            "#
            .to_string();
            RawHtml(html_shell_with_video("Server Not Found", html_content, true))
        }
    }
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

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore) -> shuttle_rocket::ShuttleRocket {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Get configuration from secrets
    let username = secrets.get("FACTORIO_USERNAME").unwrap_or_else(|| {
        eprintln!("Warning: FACTORIO_USERNAME not set, API calls will fail");
        String::new()
    });

    let token = secrets.get("FACTORIO_TOKEN").unwrap_or_else(|| {
        eprintln!("Warning: FACTORIO_TOKEN not set, API calls will fail");
        String::new()
    });

    let db_url = secrets.get("SURREAL_URL").unwrap_or_else(|| "mem://".to_string());
    let db_ns = secrets.get("SURREAL_NS").unwrap_or_else(|| "factorio".to_string());
    let db_name = secrets.get("SURREAL_DB").unwrap_or_else(|| "tracker".to_string());
    let db_user = secrets.get("SURREAL_USER");
    let db_pass = secrets.get("SURREAL_PASS");

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

    // Build Rocket server
    let rocket = rocket::build()
        .manage(app_state.db.clone())
        .manage(app_state)
        .mount("/", routes![index, server_details_page, health])
        .mount("/", routes![get_servers, get_server, get_server_history])
        .mount("/static", FileServer::from("static"));

    Ok(rocket.into())
}
