use crate::components::server_list::ServerList;
use crate::db::models::CachedServer;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone, Default)]
pub struct AppProps {
    #[prop_or_default]
    pub servers: Vec<CachedServer>,
    #[prop_or_default]
    pub error: Option<String>,
    #[prop_or_default]
    pub search: String,
    #[prop_or_default]
    pub version: String,
    #[prop_or_default]
    pub has_players: bool,
    #[prop_or_default]
    pub no_password: bool,
}

/// Root application component
#[function_component(App)]
pub fn app(props: &AppProps) -> Html {
    let total_players: usize = props.servers.iter().map(|s| s.player_count).sum();
    let servers_with_players = props.servers.iter().filter(|s| s.player_count > 0).count();

    html! {
        <div class="app">
            <header class="app-header">
                <div class="header-content">
                    <h1 class="app-title">
                        <a href="/" class="home-link" title="Home">
                            <img src="/static/favicon.svg" alt="Home" class="title-icon spinning" />
                        </a>
                        {"Factorio Server Browser"}
                    </h1>
                    <p class="app-subtitle">{"Find and explore public Factorio multiplayer servers"}</p>
                </div>
                
                <div class="global-stats">
                    <div class="stat">
                        <span class="stat-number">{props.servers.len()}</span>
                        <span class="stat-label">{"Total Servers"}</span>
                    </div>
                    <div class="stat">
                        <span class="stat-number">{servers_with_players}</span>
                        <span class="stat-label">{"Active Servers"}</span>
                    </div>
                    <div class="stat">
                        <span class="stat-number">{total_players}</span>
                        <span class="stat-label">{"Players Online"}</span>
                    </div>
                </div>
            </header>
            
            <main class="app-main">
                <ServerList 
                    servers={props.servers.clone()}
                    error={props.error.clone()}
                    current_search={props.search.clone()}
                    current_version={props.version.clone()}
                    has_players={props.has_players}
                    no_password={props.no_password}
                />
            </main>
            
            <footer class="app-footer">
                <p>{"Data from Factorio Matchmaking API • Not affiliated with Wube Software"}</p>
            </footer>
        </div>
    }
}
