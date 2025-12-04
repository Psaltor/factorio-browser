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
        <div class="server-details-page">
            <a href="/" class="back-link">{"← Back to Server List"}</a>
            
            <div class="server-details">
                <header class="details-header">
                    <h2>{parse_rich_text(&server.name)}</h2>
                    {if server.has_password {
                        html! { <span class="password-badge large">{"🔒 Password Protected"}</span> }
                    } else {
                        html! { <span class="public-badge">{"🌐 Public"}</span> }
                    }}
                </header>
                
                {if !server.description.is_empty() {
                    html! {
                        <section class="details-section">
                            <h3>{"Description"}</h3>
                            <p class="description">{parse_rich_text(&server.description)}</p>
                        </section>
                    }
                } else {
                    html! {}
                }}
                
                <section class="details-section stats-grid">
                    <div class="stat-card">
                        <span class="stat-icon">{"👥"}</span>
                        <div class="stat-info">
                            <span class="stat-value">{format!("{}/{}", server.player_count, server.max_players)}</span>
                            <span class="stat-label">{"Players"}</span>
                        </div>
                    </div>
                    
                    <div class="stat-card">
                        <span class="stat-icon">{"🎮"}</span>
                        <div class="stat-info">
                            <span class="stat-value">{&server.game_version}</span>
                            <span class="stat-label">{"Version"}</span>
                        </div>
                    </div>
                    
                    <div class="stat-card">
                        <span class="stat-icon">{"⏱️"}</span>
                        <div class="stat-info">
                            <span class="stat-value">{game_time}</span>
                            <span class="stat-label">{"Game Time"}</span>
                        </div>
                    </div>
                    
                    <div class="stat-card">
                        <span class="stat-icon">{"📦"}</span>
                        <div class="stat-info">
                            <span class="stat-value">{if server.mod_count > 0 { server.mod_count.to_string() } else { "Vanilla".to_string() }}</span>
                            <span class="stat-label">{"Mods"}</span>
                        </div>
                    </div>
                </section>
                
                {if let Some((min, max, avg)) = history_stats {
                    let chart_max = hourly_data.iter().max().copied().unwrap_or(1).max(1);
                    html! {
                        <section class="details-section">
                            <h3>{"Player Activity (Last 24h)"}</h3>
                            <div class="history-stats">
                                <div class="history-stat">
                                    <span class="history-value">{min}</span>
                                    <span class="history-label">{"Min"}</span>
                                </div>
                                <div class="history-stat">
                                    <span class="history-value">{avg}</span>
                                    <span class="history-label">{"Avg"}</span>
                                </div>
                                <div class="history-stat">
                                    <span class="history-value">{max}</span>
                                    <span class="history-label">{"Max"}</span>
                                </div>
                            </div>
                            <div class="history-chart">
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
                        <section class="details-section">
                            <h3>{"Online Players"}</h3>
                            <div class="player-list">
                                {for props.players.iter().map(|player| {
                                    html! { <span class="player-name">{player}</span> }
                                })}
                            </div>
                        </section>
                    }
                } else {
                    html! {}
                }}
                
                {if !props.mods.is_empty() {
                    html! {
                        <section class="details-section">
                            <h3>{format!("Mods ({})", props.mods.len())}</h3>
                            <div class="mods-list">
                                {for props.mods.iter().map(|m| {
                                    let mod_url = format!("https://mods.factorio.com/mod/{}", m.name);
                                    html! { 
                                        <a href={mod_url} class="mod-item" target="_blank" rel="noopener noreferrer">
                                            <span class="mod-name">{&m.name}</span>
                                            <span class="mod-version">{&m.version}</span>
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
                        <section class="details-section">
                            <h3>{"Tags"}</h3>
                            <div class="tags-list">
                                {for server.tags.iter().map(|tag| {
                                    html! { <span class="tag">{tag}</span> }
                                })}
                            </div>
                        </section>
                    }
                } else {
                    html! {}
                }}
                
                {if let Some(ref addr) = server.host_address {
                    html! {
                        <section class="details-section">
                            <h3>{"Connection"}</h3>
                            <code class="host-address">{addr}</code>
                        </section>
                    }
                } else {
                    html! {}
                }}
                
                <footer class="details-footer">
                    <span class="cached-at">{"Last updated: "}{&server.cached_at}</span>
                </footer>
            </div>
        </div>
    }
}
