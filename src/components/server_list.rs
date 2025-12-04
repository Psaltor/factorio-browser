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
        <div>
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
                    <div class="text-center py-12">
                        <div class="w-12 h-12 border-[3px] border-border-accent border-t-accent-primary rounded-full animate-spin mx-auto mb-4"></div>
                        <p>{"Loading servers..."}</p>
                    </div>
                }
            } else if let Some(ref error) = props.error {
                html! {
                    <div class="text-center py-8 bg-status-full/10 border border-status-full/30 rounded-md text-status-full">
                        <p>{"Error loading servers: "}{error}</p>
                    </div>
                }
            } else {
                html! {
                    <>
                        <div class="flex justify-between items-center flex-wrap gap-4 mb-4 text-text-secondary text-sm">
                            <span>{format!("Showing {} of {} servers", filtered_servers.len(), props.servers.len())}</span>
                            
                            <div class="flex items-center gap-2">
                                <span class="text-text-muted text-[0.85rem]">{"Sort by:"}</span>
                                <button type="button" class="sort-button active py-1 px-2 bg-bg-inset border border-border-subtle rounded-sm text-text-secondary font-display text-[0.85rem] cursor-pointer transition-all duration-200 hover:border-accent-primary hover:text-accent-primary" data-sort="players" data-dir="desc">
                                    {"Players "}<span class="sort-arrow text-xs ml-0.5">{"▼"}</span>
                                </button>
                                <button type="button" class="sort-button py-1 px-2 bg-bg-inset border border-border-subtle rounded-sm text-text-secondary font-display text-[0.85rem] cursor-pointer transition-all duration-200 hover:border-accent-primary hover:text-accent-primary" data-sort="time">
                                    {"Game Time "}<span class="sort-arrow text-xs ml-0.5">{""}</span>
                                </button>
                                
                                <div class="flex gap-0.5 ml-4 pl-4 border-l border-border-subtle">
                                    <button type="button" class="view-btn active py-1 px-2 bg-bg-inset border border-border-subtle text-text-secondary text-base cursor-pointer transition-all duration-200 leading-none rounded-l-sm hover:border-accent-primary hover:text-accent-primary" data-view="grid" title="Grid view">{"▦"}</button>
                                    <button type="button" class="view-btn py-1 px-2 bg-bg-inset border border-border-subtle border-l-0 text-text-secondary text-base cursor-pointer transition-all duration-200 leading-none rounded-r-sm hover:border-accent-primary hover:text-accent-primary" data-view="list" title="List view">{"☰"}</button>
                                </div>
                            </div>
                        </div>
                        
                        <div class="server-grid grid grid-cols-[repeat(auto-fill,minmax(320px,1fr))] gap-6">
                            <div class="list-header hidden items-center gap-4 py-2 px-4 bg-bg-inset border border-border-subtle rounded-sm sticky top-0 z-10 text-xs font-semibold uppercase tracking-widest text-text-secondary">
                                <span class="flex-1 min-w-0">{"Name"}</span>
                                <span class="w-[60px] text-center">{"Players"}</span>
                                <span class="w-[70px] text-center">{"Version"}</span>
                                <span class="w-[80px] text-center">{"Time"}</span>
                                <span class="w-[80px] text-right">{"Mods"}</span>
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
                                <div class="text-center py-12 text-text-muted">
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
