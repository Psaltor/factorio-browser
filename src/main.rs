use factorio_browser::api::factorio::{ApiError, FactorioClient};
// TODO: Re-enable API routes later
// use factorio_browser::api::routes::{get_server, get_server_history, get_servers, health};
use factorio_browser::components::app::{App, AppProps};
use factorio_browser::components::server_details::ServerDetails;
use factorio_browser::db::models::CachedServer;
use factorio_browser::db::queries::DbClient;
use factorio_browser::utils::strip_all_tags;
use rocket::Request;
use rocket::fairing::AdHoc;
use rocket::form::FromForm;
use rocket::fs::{FileServer, NamedFile};
use rocket::http::{Header, Status};
use rocket::response::content::RawHtml;
use rocket::response::{Responder, Response};
use rocket::{State, get, routes};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use yew::ServerRenderer;

/// Application state
struct AppState {
    db: Arc<DbClient>,
    factorio_client: Arc<FactorioClient>,
    last_error: Arc<RwLock<Option<String>>>,
    // Add cached servers
    cached_servers: Arc<RwLock<Vec<CachedServer>>>,
    server_details:
        Arc<RwLock<HashMap<u64, (Instant, factorio_browser::api::factorio::GameDetails)>>>,
    detail_requests: Arc<Semaphore>,
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
    let title_text = html_escape::encode_text(title);
    let title_attribute = html_escape::encode_double_quoted_attribute(title);

    let video_element = if with_video {
        format!(
            r#"<video class="video-background" autoplay muted loop playsinline preload="auto">
        <source src="{}" type="video/mp4">
    </video>"#,
            video_url
        )
    } else {
        String::new()
    };

    let body_class = if with_video {
        " class=\"has-video\""
    } else {
        ""
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title_text}</title>
    <meta name="description" content="Find and explore public Factorio multiplayer servers. Browse servers by version, tags, player count, and more.">
    <meta name="keywords" content="Factorio, multiplayer, servers, server browser, gaming, factory">
    <meta name="author" content="lambs.cafe">
    <meta name="theme-color" content="#0d0d0f">
    
    <!-- Open Graph / Facebook -->
    <meta property="og:type" content="website">
    <meta property="og:title" content="{title_attribute}">
    <meta property="og:description" content="Find and explore public Factorio multiplayer servers. Browse servers by version, tags, player count, and more.">
    <meta property="og:image" content="/static/favicon.svg">
    <meta property="og:site_name" content="Factorio Server Browser">
    
    <!-- Twitter -->
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="{title_attribute}">
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
        title_text = title_text,
        title_attribute = title_attribute,
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

    RawHtml(html_shell_with_video(
        "Factorio Server Browser",
        html_content,
        true,
    ))
}

/// Server details page
#[get("/server/<game_id>")]
async fn server_details_page(
    state: &State<Arc<AppState>>,
    game_id: u64,
) -> Result<RawHtml<String>, (Status, RawHtml<String>)> {
    let known_server = state
        .cached_servers
        .read()
        .await
        .iter()
        .any(|server| server.game_id == game_id);
    let cached_details = state
        .server_details
        .read()
        .await
        .get(&game_id)
        .filter(|(cached_at, _)| cached_at.elapsed() < Duration::from_secs(30))
        .map(|(_, details)| details.clone());

    let api_result = if !known_server {
        Err(ApiError::HttpStatus(reqwest::StatusCode::NOT_FOUND))
    } else if let Some(details) = cached_details {
        Ok(details)
    } else {
        let _permit = state
            .detail_requests
            .acquire()
            .await
            .expect("semaphore is open");
        let result = state.factorio_client.get_game_details(game_id).await;
        if let Ok(details) = &result {
            let mut cache = state.server_details.write().await;
            cache.retain(|_, (cached_at, _)| cached_at.elapsed() < Duration::from_secs(30));
            cache.insert(game_id, (Instant::now(), details.clone()));
        }
        result
    };

    // Fetch raw history and preserve gaps where no observation was recorded.
    let raw_history = state
        .db
        .get_server_history(game_id, 24)
        .await
        .unwrap_or_default();

    let history = fill_history_gaps(raw_history);

    match api_result {
        Ok(details) => {
            let title = format!(
                "{} - Factorio Server Browser",
                strip_all_tags(&details.name)
            );

            let props = factorio_browser::components::server_details::ServerDetailsProps {
                server: details,
                history,
            };
            let renderer = ServerRenderer::<ServerDetails>::with_props(move || props.clone());
            let html_content = renderer.render().await;
            Ok(RawHtml(html_shell_with_video(&title, html_content, true)))
        }
        Err(error) => {
            let (status, heading, message) = match error {
                ApiError::HttpStatus(reqwest::StatusCode::NOT_FOUND) => (
                    Status::NotFound,
                    "Server Not Found",
                    "The requested server could not be found. It may have restarted and received a new game ID.",
                ),
                ApiError::RequestFailed(error) if error.is_timeout() => (
                    Status::GatewayTimeout,
                    "Factorio API Timed Out",
                    "The server details service did not respond in time. Please try again shortly.",
                ),
                _ => (
                    Status::BadGateway,
                    "Server Details Unavailable",
                    "Server details are temporarily unavailable. Please try again shortly.",
                ),
            };
            let html_content = format!(
                r#"
                <div class="min-h-screen flex flex-col">
                    <header class="bg-bg-card/65 backdrop-blur-[10px] border-b border-border-subtle py-8 px-6">
                        <div class="max-w-[1400px] mx-auto text-center">
                            <h1 class="text-4xl font-bold text-text-bright">{heading}</h1>
                        </div>
                    </header>
                    <main class="flex-1 max-w-[1400px] mx-auto py-8 px-6 w-full">
                        <div class="text-center py-8 bg-status-full/10 border border-status-full/30 rounded-md text-status-full">
                            <p class="mb-4">{message}</p>
                            <a href="/" class="text-accent-primary hover:text-accent-secondary transition-colors duration-200">
                                ← Back to Server List
                            </a>
                        </div>
                    </main>
                </div>
            "#
            );
            Err((
                status,
                RawHtml(html_shell_with_video(heading, html_content, true)),
            ))
        }
    }
}

/// Wrapper for NamedFile that adds caching headers
pub struct CachedFile(NamedFile);

impl<'r> Responder<'r, 'static> for CachedFile {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        Response::build_from(self.0.respond_to(req)?)
            // Cache for 1 day, revalidate with server
            .header(Header::new(
                "Cache-Control",
                "public, max-age=86400, must-revalidate",
            ))
            .ok()
    }
}

/// Fill gaps in history data while distinguishing missing observations from zero players.
fn fill_history_gaps(
    raw_history: Vec<factorio_browser::db::models::ServerHistory>,
) -> Vec<factorio_browser::components::server_details::HistoryEntry> {
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
            if (0..24).contains(&hours_ago) {
                hourly_counts
                    .entry(hours_ago)
                    .or_default()
                    .push(record.player_count);
            }
        }
    }

    // Generate 24 hourly entries (newest first to match expected order)
    // Each entry represents the average player count for that hour, or None if no data.
    (0..24)
        .map(|hours_ago| {
            let avg_count = hourly_counts
                .get(&hours_ago)
                .map(|counts| counts.iter().sum::<usize>() / counts.len().max(1));

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
                        // Update in-memory cache from DB
                        match state.db.get_all_servers().await {
                            Ok(all_servers) => {
                                *state.cached_servers.write().await = all_servers;
                                *state.last_error.write().await = None;
                            }
                            Err(error) => {
                                eprintln!("Failed to reload server cache: {error}");
                                *state.last_error.write().await = Some(
                                    "Server data was updated, but the displayed cache may be stale."
                                        .to_string(),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        let raw_msg = format!("Failed to cache servers: {}", e);
                        eprintln!("{}", raw_msg);
                        // Display sanitized message to users
                        *state.last_error.write().await =
                            Some("Failed to update server cache.".to_string());
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
#[allow(clippy::result_large_err)]
async fn main() -> Result<(), rocket::Error> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Get configuration from environment variables
    let username = required_env("FACTORIO_USERNAME");
    let token = required_env("FACTORIO_TOKEN");

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

    let cached_servers = match db.get_all_servers().await {
        Ok(servers) => {
            println!("Loaded {} servers from cache", servers.len());
            servers
        }
        Err(error) => {
            eprintln!("Failed to load server cache: {error}");
            Vec::new()
        }
    };

    // Create application state with the persistent cache available immediately.
    let app_state = Arc::new(AppState {
        db: db.clone(),
        factorio_client: factorio_client.clone(),
        last_error: Arc::new(RwLock::new(None)),
        cached_servers: Arc::new(RwLock::new(cached_servers)),
        server_details: Arc::new(RwLock::new(HashMap::new())),
        detail_requests: Arc::new(Semaphore::new(8)),
    });

    // Start background refresh task
    let refresh_state = app_state.clone();
    tokio::spawn(async move {
        refresh_servers(refresh_state).await;
    });

    let static_dir = std::env::var_os("STATIC_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .expect("Cannot get current directory")
                .join("static")
        });
    assert!(
        static_dir.is_dir(),
        "Static asset directory does not exist: {}",
        static_dir.display()
    );

    // Build and launch Rocket server
    rocket::build()
        .manage(app_state.db.clone())
        .manage(app_state)
        .attach(security_headers())
        .mount("/", routes![index, server_details_page])
        .mount("/static", FileServer::from(static_dir))
        // TODO: Re-enable API routes later
        // .mount("/", routes![health, get_servers, get_server, get_server_history])
        .launch()
        .await?;

    Ok(())
}

fn security_headers() -> AdHoc {
    AdHoc::on_response("Security headers", |_request, response| {
        Box::pin(async move {
            response.set_raw_header("X-Content-Type-Options", "nosniff");
            response.set_raw_header("X-Frame-Options", "DENY");
            response.set_raw_header("Referrer-Policy", "strict-origin-when-cross-origin");
            response.set_raw_header(
                "Permissions-Policy",
                "camera=(), geolocation=(), microphone=()",
            );
            response.set_raw_header(
                "Content-Security-Policy",
                "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src https://fonts.gstatic.com; img-src 'self' data:; media-src https://lambs.cafe; object-src 'none'; base-uri 'self'; frame-ancestors 'none'",
            );
        })
    })
}

fn required_env(name: &str) -> String {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| panic!("{name} must be set and non-empty"))
}

#[cfg(test)]
mod tests {
    use super::{fill_history_gaps, html_shell_with_video};
    use factorio_browser::db::models::ServerHistory;

    #[test]
    fn escapes_untrusted_page_titles() {
        let shell = html_shell_with_video(
            "server </title><script>alert('xss')</script> \"quoted\"",
            "<main>safe content</main>".to_string(),
            false,
        );

        assert!(!shell.contains("</title><script>"));
        assert!(shell.contains("<title>server &lt;/title&gt;&lt;script&gt;alert('xss')&lt;/script&gt; \"quoted\"</title>"));
        assert!(shell.contains("content=\"server &lt;/title&gt;&lt;script&gt;alert('xss')&lt;/script&gt; &quot;quoted&quot;\""));
        assert!(shell.contains("<main>safe content</main>"));
    }

    #[test]
    fn history_distinguishes_zero_players_from_missing_observations() {
        let history = fill_history_gaps(vec![ServerHistory {
            id: None,
            game_id: 1,
            player_count: 0,
            recorded_at: chrono::Utc::now().to_rfc3339(),
        }]);

        assert_eq!(history.len(), 24);
        assert_eq!(history[0].player_count, Some(0));
        assert!(
            history[1..]
                .iter()
                .all(|entry| entry.player_count.is_none())
        );
    }
}
