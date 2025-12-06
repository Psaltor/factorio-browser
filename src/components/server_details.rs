use crate::components::footer::Footer;
use crate::db::models::CachedServer;
use crate::utils::parse_rich_text;
use yew::prelude::*;

/// Player count history entry for display
#[derive(Clone, PartialEq)]
pub struct HistoryEntry {
    pub player_count: usize,
    pub recorded_at: String,
}

/// Mod info for display
#[derive(Clone, PartialEq)]
pub struct ModEntry {
    pub name: String,
    pub version: String,
}

#[derive(Properties, PartialEq, Clone)]
pub struct ServerDetailsProps {
    pub server: CachedServer,
    #[prop_or_default]
    pub history: Vec<HistoryEntry>,
    #[prop_or_default]
    pub players: Vec<String>,
    #[prop_or_default]
    pub mods: Vec<ModEntry>,
}

/// Detailed server view component (SSR-compatible, standalone page)
#[function_component(ServerDetails)]
pub fn server_details(props: &ServerDetailsProps) -> Html {
    let server = &props.server;

    // Format game time (API returns minutes)
    let total_minutes = server.game_time_elapsed;
    let days = total_minutes / (60 * 24);
    let hours = (total_minutes % (60 * 24)) / 60;
    let minutes = total_minutes % 60;

    let game_time = if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else {
        format!("{}h {}m", hours, minutes)
    };

    // Calculate history stats and aggregate into 24 hourly buckets
    let (history_stats, hourly_data) = if !props.history.is_empty() {
        let counts: Vec<usize> = props.history.iter().map(|h| h.player_count).collect();
        let max = *counts.iter().max().unwrap_or(&0);
        let min = *counts.iter().min().unwrap_or(&0);
        let avg = counts.iter().sum::<usize>() / counts.len();
        
        // Aggregate into 24 hourly buckets (newest first in history)
        // Each bucket represents ~60 data points (1 per minute for 1 hour)
        let bucket_size = (props.history.len() / 24).max(1);
        let hourly: Vec<usize> = props.history
            .chunks(bucket_size)
            .take(24)
            .map(|chunk| {
                // Average of the chunk
                chunk.iter().map(|h| h.player_count).sum::<usize>() / chunk.len().max(1)
            })
            .collect();
        
        (Some((min, max, avg)), hourly)
    } else {
        (None, Vec::new())
    };

    html! {
        <div class="min-h-screen py-8 px-6 max-w-[800px] mx-auto">
            <a href="/" class="inline-block text-accent-primary no-underline mb-6 text-[0.95rem] transition-colors duration-200 hover:text-accent-secondary">{"← Back to Server List"}</a>
            
            <div class="bg-bg-card/65 backdrop-blur-[10px] border border-border-subtle rounded-lg max-w-[700px] w-full max-h-[90vh] overflow-y-auto relative animate-slide-up">
                <header class="p-8 pb-6 border-b border-border-subtle">
                    <h2 class="text-2xl mb-2 pr-12 break-words break-all">{parse_rich_text(&server.name)}</h2>
                    {if server.has_password {
                        html! { <span class="inline-block py-1 px-2 rounded-sm text-[0.85rem] bg-status-full/15 text-status-full">{"🔒 Password Protected"}</span> }
                    } else {
                        html! { <span class="inline-block py-1 px-2 rounded-sm text-[0.85rem] bg-status-low/15 text-status-low">{"🌐 Public"}</span> }
                    }}
                </header>
                
                {if !server.description.is_empty() {
                    html! {
                        <section class="p-6 px-8 border-b border-border-subtle">
                            <h3 class="text-[0.85rem] text-text-secondary uppercase tracking-wider mb-4">{"Description"}</h3>
                            <p class="text-text-primary leading-relaxed">{parse_rich_text(&server.description)}</p>
                        </section>
                    }
                } else {
                    html! {}
                }}
                
                <section class="p-6 px-8 border-b border-border-subtle grid grid-cols-2 gap-4 max-md:grid-cols-1">
                    <div class="flex items-center gap-4 p-4 bg-bg-inset border border-border-subtle rounded-sm">
                        <span class="text-2xl">{"👥"}</span>
                        <div class="flex flex-col">
                            <span class="text-lg font-semibold font-mono text-accent-primary">{format!("{}/{}", server.player_count, server.max_players)}</span>
                            <span class="text-xs text-text-secondary">{"Players"}</span>
                        </div>
                    </div>
                    
                    <div class="flex items-center gap-4 p-4 bg-bg-inset border border-border-subtle rounded-sm">
                        <span class="text-2xl">{"🎮"}</span>
                        <div class="flex flex-col">
                            <span class="text-lg font-semibold font-mono text-accent-primary">{&server.game_version}</span>
                            <span class="text-xs text-text-secondary">{"Version"}</span>
                        </div>
                    </div>
                    
                    <div class="flex items-center gap-4 p-4 bg-bg-inset border border-border-subtle rounded-sm">
                        <span class="text-2xl">{"⏱️"}</span>
                        <div class="flex flex-col">
                            <span class="text-lg font-semibold font-mono text-accent-primary">{game_time}</span>
                            <span class="text-xs text-text-secondary">{"Game Time"}</span>
                        </div>
                    </div>
                    
                    <div class="flex items-center gap-4 p-4 bg-bg-inset border border-border-subtle rounded-sm">
                        <span class="text-2xl">{"📦"}</span>
                        <div class="flex flex-col">
                            <span class="text-lg font-semibold font-mono text-accent-primary">{if server.mod_count > 0 { server.mod_count.to_string() } else { "Vanilla".to_string() }}</span>
                            <span class="text-xs text-text-secondary">{"Mods"}</span>
                        </div>
                    </div>
                </section>
                
                {if let Some((min, max, avg)) = history_stats {
                    let chart_max = hourly_data.iter().max().copied().unwrap_or(1).max(1);
                    html! {
                        <section class="p-6 px-8 border-b border-border-subtle">
                            <h3 class="text-[0.85rem] text-text-secondary uppercase tracking-wider mb-4">{"Player Activity (Last 24h)"}</h3>
                            <div class="flex gap-6 mb-6">
                                <div class="text-center p-4 bg-bg-dark rounded-md flex-1">
                                    <span class="block text-2xl font-semibold font-mono text-accent-primary">{min}</span>
                                    <span class="text-xs text-text-secondary uppercase tracking-wider">{"Min"}</span>
                                </div>
                                <div class="text-center p-4 bg-bg-dark rounded-md flex-1">
                                    <span class="block text-2xl font-semibold font-mono text-accent-primary">{avg}</span>
                                    <span class="text-xs text-text-secondary uppercase tracking-wider">{"Avg"}</span>
                                </div>
                                <div class="text-center p-4 bg-bg-dark rounded-md flex-1">
                                    <span class="block text-2xl font-semibold font-mono text-accent-primary">{max}</span>
                                    <span class="text-xs text-text-secondary uppercase tracking-wider">{"Max"}</span>
                                </div>
                            </div>
                            <div class="flex items-end gap-0.5 h-20 p-2 bg-bg-inset rounded-md">
                                {for hourly_data.iter().rev().map(|&count| {
                                    let height = (count as f32 / chart_max as f32 * 100.0) as u32;
                                    let height_style = format!("height: {}%", height.max(2));
                                    html! {
                                        <div class="history-bar" style={height_style} title={format!("{} players (avg)", count)}></div>
                                    }
                                })}
                            </div>
                        </section>
                    }
                } else {
                    html! {}
                }}
                
                {if !props.players.is_empty() {
                    html! {
                        <section class="p-6 px-8 border-b border-border-subtle">
                            <h3 class="text-[0.85rem] text-text-secondary uppercase tracking-wider mb-4">{"Online Players"}</h3>
                            <div class="flex flex-wrap gap-2">
                                {for props.players.iter().map(|player| {
                                    html! { <span class="py-1 px-2 bg-bg-dark border border-border-accent rounded-sm text-sm font-mono">{player}</span> }
                                })}
                            </div>
                        </section>
                    }
                } else {
                    html! {}
                }}
                
                {if !props.mods.is_empty() {
                    html! {
                        <section class="p-6 px-8 border-b border-border-subtle">
                            <h3 class="text-[0.85rem] text-text-secondary uppercase tracking-wider mb-4">{"Mods"}</h3>
                            <div class="mods-list grid grid-cols-[repeat(auto-fill,minmax(250px,1fr))] gap-2 max-h-[400px] overflow-y-auto">
                                {for props.mods.iter().map(|m| {
                                    let mod_url = format!("https://mods.factorio.com/mod/{}", m.name);
                                    html! { 
                                        <a href={mod_url} class="flex justify-between items-center py-1 px-2 bg-bg-inset border border-border-subtle rounded-sm text-[0.85rem] no-underline transition-all duration-200 hover:border-accent-primary hover:bg-bg-card" target="_blank" rel="noopener noreferrer">
                                            <span class="text-accent-primary overflow-hidden text-ellipsis whitespace-nowrap hover:text-accent-secondary">{&m.name}</span>
                                            <span class="text-text-muted font-mono text-xs ml-2 flex-shrink-0">{&m.version}</span>
                                        </a>
                                    }
                                })}
                            </div>
                        </section>
                    }
                } else {
                    html! {}
                }}
                
                {if !server.tags.is_empty() {
                    html! {
                        <section class="p-6 px-8 border-b border-border-subtle">
                            <h3 class="text-[0.85rem] text-text-secondary uppercase tracking-wider mb-4">{"Tags"}</h3>
                            <div class="flex flex-wrap gap-2">
                                {for server.tags.iter().map(|tag| {
                                    html! { <span class="py-1 px-2 bg-accent-glow border border-accent-primary rounded-sm text-xs text-accent-primary">{parse_rich_text(tag)}</span> }
                                })}
                            </div>
                        </section>
                    }
                } else {
                    html! {}
                }}
                
                {if let Some(ref addr) = server.host_address {
                    let join_url = format!("steam://run/427520//--mp-connect%20{}", addr);
                    html! {
                        <section class="p-6 px-8 border-b border-border-subtle">
                            <h3 class="text-[0.85rem] text-text-secondary uppercase tracking-wider mb-4">{"Connection"}</h3>
                            <div class="flex items-center gap-4">
                                <code class="flex-1 p-4 bg-bg-dark rounded-sm font-mono text-sm text-accent-primary break-all">{addr}</code>
                                <a href={join_url} class="py-2 px-6 bg-btn-green border border-btn-green-dark rounded-sm text-bg-dark font-display text-[0.95rem] font-semibold cursor-pointer transition-all duration-200 hover:bg-btn-green-hover active:bg-btn-green-dark no-underline">
                                    {"Join"}
                                </a>
                            </div>
                        </section>
                    }
                } else {
                    html! {}
                }}
                <div class="p-4 px-8 bg-bg-dark rounded-b-lg">
                    <Footer />
                </div>
            </div>
        </div>
    }
}
