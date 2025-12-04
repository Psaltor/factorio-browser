#[macro_use]
extern crate rocket;

use factory_tracker::api::factorio::FactorioClient;
use factory_tracker::api::routes::{get_server, get_server_history, get_servers, health};
use factory_tracker::components::app::{App, AppProps};
use factory_tracker::components::server_details::ServerDetails;
use factory_tracker::db::queries::DbClient;
use rocket::form::FromForm;
use rocket::fs::FileServer;
use rocket::response::content::RawHtml;
use rocket::State;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use yew::ServerRenderer;

/// Application state
struct AppState {
    db: Arc<DbClient>,
    factorio_client: Arc<FactorioClient>,
    last_error: Arc<RwLock<Option<String>>>,
}

/// Query parameters for the main page
#[derive(Debug, FromForm, Default)]
struct IndexFilters {
    search: Option<String>,
    version: Option<String>,
    has_players: Option<bool>,
    no_password: Option<bool>,
}

/// Wrap HTML content with the page shell
fn html_shell(title: &str, content: String) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <link rel="stylesheet" href="/static/style.css">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600&family=Outfit:wght@300;400;500;600;700&display=swap" rel="stylesheet">
</head>
<body>
    {content}
</body>
</html>"#,
        title = title,
        content = content
    )
}

/// Main SSR route - renders the Yew app to HTML
#[get("/?<filters..>")]
async fn index(state: &State<Arc<AppState>>, filters: IndexFilters) -> RawHtml<String> {
    let servers = state.db.get_all_servers().await.unwrap_or_default();
    let error = state.last_error.read().await.clone();

    let props = AppProps {
        servers,
        error,
        search: filters.search.unwrap_or_default(),
        version: filters.version.unwrap_or_default(),
        has_players: filters.has_players.unwrap_or(false),
        no_password: filters.no_password.unwrap_or(false),
    };

    let renderer = ServerRenderer::<App>::with_props(move || props.clone());
    let html_content = renderer.render().await;

    RawHtml(html_shell("Factorio Server Browser", html_content))
}

/// Server details page
#[get("/server/<game_id>")]
async fn server_details_page(state: &State<Arc<AppState>>, game_id: u64) -> RawHtml<String> {
    use factory_tracker::components::server_details::HistoryEntry;
    
    let server = state.db.get_server(game_id).await.ok().flatten();
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
            let title = format!("{} - Factorio Server Browser", server.name);
            let props = factory_tracker::components::server_details::ServerDetailsProps { 
                server, 
                history,
            };
            let renderer = ServerRenderer::<ServerDetails>::with_props(move || props.clone());
            let html_content = renderer.render().await;
            RawHtml(html_shell(&title, html_content))
        }
        None => {
            let html_content = r#"
                <div class="app">
                    <header class="app-header">
                        <div class="header-content">
                            <h1 class="app-title">Server Not Found</h1>
                        </div>
                    </header>
                    <main class="app-main">
                        <div class="error">
                            <p>The requested server could not be found.</p>
                            <a href="/">← Back to Server List</a>
                        </div>
                    </main>
                </div>
            "#
            .to_string();
            RawHtml(html_shell("Server Not Found", html_content))
        }
    }
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

                // Cache the servers
                match state.db.cache_servers(servers).await {
                    Ok(_) => {
                        println!("Cached {} servers", count);
                        *state.last_error.write().await = None;
                    }
                    Err(e) => {
                        let msg = format!("Failed to cache servers: {}", e);
                        eprintln!("{}", msg);
                        *state.last_error.write().await = Some(msg);
                    }
                }

                // Clean up old history
                if let Err(e) = state.db.cleanup_old_history().await {
                    eprintln!("Failed to cleanup history: {}", e);
                }
            }
            Err(e) => {
                let msg = format!("Failed to fetch servers: {}", e);
                eprintln!("{}", msg);
                *state.last_error.write().await = Some(msg);
            }
        }

        // Wait before next refresh (60 seconds)
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

#[launch]
async fn rocket() -> _ {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Get configuration from environment
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
    let db_name = std::env::var("SURREAL_DB").unwrap_or_else(|_| "tracker".to_string());

    // Initialize database
    let db = DbClient::connect(&db_url, &db_ns, &db_name)
        .await
        .expect("Failed to connect to database");

    let db = Arc::new(db);

    // Initialize Factorio API client
    let factorio_client = FactorioClient::new_shared(username, token);

    // Create application state
    let app_state = Arc::new(AppState {
        db: db.clone(),
        factorio_client: factorio_client.clone(),
        last_error: Arc::new(RwLock::new(None)),
    });

    // Start background refresh task
    let refresh_state = app_state.clone();
    tokio::spawn(async move {
        refresh_servers(refresh_state).await;
    });

    // Build Rocket server
    rocket::build()
        .manage(app_state.db.clone())
        .manage(app_state)
        .mount("/", routes![index, server_details_page, health])
        .mount("/", routes![get_servers, get_server, get_server_history])
        .mount("/static", FileServer::from("static"))
}
