use crate::components::filters::Filters;
use crate::components::server_card::ServerCard;
use crate::db::models::CachedServer;
use semver::Version;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct ServerListProps {
    pub servers: Vec<CachedServer>,
    #[prop_or_default]
    pub loading: bool,
    #[prop_or_default]
    pub error: Option<String>,
    #[prop_or_default]
    pub current_search: String,
    #[prop_or_default]
    pub current_version: String,
    #[prop_or_default]
    pub has_players: bool,
    #[prop_or_default]
    pub no_password: bool,
}

/// Server list component with filtering (SSR-compatible)
#[function_component(ServerList)]
pub fn server_list(props: &ServerListProps) -> Html {
    // Extract unique versions from servers, sorted by semver (descending)
    let mut versions: Vec<String> = props
        .servers
        .iter()
        .map(|s| s.game_version.clone())
        .collect();
    versions.sort_by(|a, b| {
        let va = Version::parse(a).ok();
        let vb = Version::parse(b).ok();
        vb.cmp(&va) // Descending order
    });
    versions.dedup();

    // Latest version is first after sorting
    let latest_version = versions.first().cloned().unwrap_or_default();

    // Determine effective version filter (empty = latest, "all" = no filter)
    let effective_version = if props.current_version.is_empty() {
        &latest_version
    } else if props.current_version == "all" {
        ""
    } else {
        &props.current_version
    };

    // Filter servers based on props
    let filtered_servers: Vec<&CachedServer> = props
        .servers
        .iter()
        .filter(|s| {
            // Search filter
            if !props.current_search.is_empty() {
                let search_lower = props.current_search.to_lowercase();
                if !s.name.to_lowercase().contains(&search_lower)
                    && !s.description.to_lowercase().contains(&search_lower)
                {
                    return false;
                }
            }

            // Version filter
            if !effective_version.is_empty() && !s.game_version.starts_with(effective_version) {
                return false;
            }

            // Has players filter
            if props.has_players && s.player_count == 0 {
                return false;
            }

            // No password filter
            if props.no_password && s.has_password {
                return false;
            }

            true
        })
        .collect();

    html! {
        <div class="server-list-container">
            <Filters 
                current_search={props.current_search.clone()}
                current_version={props.current_version.clone()}
                has_players={props.has_players}
                no_password={props.no_password}
                versions={versions}
                latest_version={latest_version}
            />
            
            {if props.loading {
                html! {
                    <div class="loading">
                        <div class="spinner"></div>
                        <p>{"Loading servers..."}</p>
                    </div>
                }
            } else if let Some(ref error) = props.error {
                html! {
                    <div class="error">
                        <p>{"Error loading servers: "}{error}</p>
                    </div>
                }
            } else {
                html! {
                    <>
                        <div class="server-stats">
                            <span>{format!("Showing {} of {} servers", filtered_servers.len(), props.servers.len())}</span>
                            
                            <div class="sort-bar">
                                <span class="sort-label">{"Sort by:"}</span>
                                <button type="button" class="sort-button active" data-sort="players" data-dir="desc">
                                    {"Players "}<span class="sort-arrow">{"▼"}</span>
                                </button>
                                <button type="button" class="sort-button" data-sort="time">
                                    {"Game Time "}<span class="sort-arrow">{""}</span>
                                </button>
                                
                                <div class="view-toggle">
                                    <button type="button" class="view-btn active" data-view="grid" title="Grid view">{"▦"}</button>
                                    <button type="button" class="view-btn" data-view="list" title="List view">{"☰"}</button>
                                </div>
                            </div>
                        </div>
                        
                        <div class="server-grid">
                            <div class="list-header">
                                <span class="header-name">{"Name"}</span>
                                <span class="header-players">{"Players"}</span>
                                <span class="header-version">{"Version"}</span>
                                <span class="header-time">{"Time"}</span>
                                <span class="header-mods">{"Mods"}</span>
                            </div>
                            {for filtered_servers.iter().map(|server| {
                                html! {
                                    <ServerCard 
                                        server={(*server).clone()} 
                                    />
                                }
                            })}
                        </div>
                        
                        {if filtered_servers.is_empty() {
                            html! {
                                <div class="no-results">
                                    <p>{"No servers match your filters"}</p>
                                </div>
                            }
                        } else {
                            html! {}
                        }}
                    </>
                }
            }}
        </div>
    }
}
