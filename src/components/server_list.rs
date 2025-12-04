use crate::components::filters::Filters;
use crate::components::server_card::ServerCard;
use crate::db::models::CachedServer;
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
    // Extract unique versions from servers
    let versions: Vec<String> = {
        let mut vers: Vec<String> = props
            .servers
            .iter()
            .map(|s| s.game_version.clone())
            .collect();
        vers.sort();
        vers.dedup();
        vers
    };

    // Filter servers based on props (filtering done server-side in SSR)
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
            if !props.current_version.is_empty() && !s.game_version.starts_with(&props.current_version) {
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
                        </div>
                        
                        <div class="server-grid">
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
