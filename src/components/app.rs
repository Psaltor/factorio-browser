use crate::components::server_list::ServerList;
use crate::db::models::CachedServer;
use chrono::Datelike;
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
    let current_year = chrono::Utc::now().year();

    html! {
        <div class="min-h-screen flex flex-col">
            <header class="bg-bg-card/65 backdrop-blur-[10px] border-b border-border-subtle py-8 px-6">
                <div class="max-w-[1400px] mx-auto text-center mb-6">
                    <a href="/" class="inline-block" title="Home">
                        <img src="https://cdn.factorio.com/assets/img/web/factorio-logo.png" alt="Factorio" class="h-16 mx-auto" />
                    </a>
                    <h1 class="text-3xl font-bold text-text-bright mt-2">{"Server Browser"}</h1>
                    <p class="text-text-secondary text-lg mt-2">{"Find and explore public Factorio multiplayer servers"}</p>
                    <p class="text-text-muted text-sm mt-1">{"Not affiliated with Wube Software"}</p>
                </div>
                
                <div class="flex justify-center gap-8 flex-wrap">
                    <div class="text-center py-4 px-6 bg-bg-card border border-border-subtle rounded-sm min-w-[140px]">
                        <span class="block text-[2rem] font-semibold text-accent-primary font-mono">{props.servers.len()}</span>
                        <span class="block text-[0.85rem] text-text-secondary uppercase tracking-wider">{"Total Servers"}</span>
                    </div>
                    <div class="text-center py-4 px-6 bg-bg-card border border-border-subtle rounded-sm min-w-[140px]">
                        <span class="block text-[2rem] font-semibold text-accent-primary font-mono">{servers_with_players}</span>
                        <span class="block text-[0.85rem] text-text-secondary uppercase tracking-wider">{"Active Servers"}</span>
                    </div>
                    <div class="text-center py-4 px-6 bg-bg-card border border-border-subtle rounded-sm min-w-[140px]">
                        <span class="block text-[2rem] font-semibold text-accent-primary font-mono">{total_players}</span>
                        <span class="block text-[0.85rem] text-text-secondary uppercase tracking-wider">{"Players Online"}</span>
                    </div>
                </div>
            </header>
            
            <main class="flex-1 max-w-[1400px] mx-auto py-8 px-6 w-full">
                <ServerList 
                    servers={props.servers.clone()}
                    error={props.error.clone()}
                    current_search={props.search.clone()}
                    current_version={props.version.clone()}
                    has_players={props.has_players}
                    no_password={props.no_password}
                />
            </main>
            
            <footer class="text-center p-6 text-text-muted text-sm border-t border-border-subtle">
                <p>{format!("© {} • Brought to you by ", current_year)}<a href="https://lambs.cafe" class="text-accent-primary hover:text-accent-secondary transition-colors" target="_blank" rel="noopener noreferrer">{"lambs.cafe"}</a></p>
                <p class="mt-1">{"Data from Factorio Matchmaking API • Not affiliated with Wube Software"}</p>
            </footer>
        </div>
    }
}
