use crate::db::models::CachedServer;
use yew::prelude::*;

/// Player count history entry for display
#[derive(Clone, PartialEq)]
pub struct HistoryEntry {
    pub player_count: usize,
    pub recorded_at: String,
}

#[derive(Properties, PartialEq, Clone)]
pub struct ServerDetailsProps {
    pub server: CachedServer,
    #[prop_or_default]
    pub history: Vec<HistoryEntry>,
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

    // Calculate history stats
    let history_stats = if !props.history.is_empty() {
        let counts: Vec<usize> = props.history.iter().map(|h| h.player_count).collect();
        let max = *counts.iter().max().unwrap_or(&0);
        let min = *counts.iter().min().unwrap_or(&0);
        let avg = counts.iter().sum::<usize>() / counts.len();
        Some((min, max, avg))
    } else {
        None
    };

    html! {
        <div class="server-details-page">
            <a href="/" class="back-link">{"← Back to Server List"}</a>
            
            <div class="server-details">
                <header class="details-header">
                    <h2>{&server.name}</h2>
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
                            <p class="description">{&server.description}</p>
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
                                {for props.history.iter().rev().take(24).map(|h| {
                                    let height = if max > 0 {
                                        (h.player_count as f32 / max as f32 * 100.0) as u32
                                    } else {
                                        0
                                    };
                                    let height_style = format!("height: {}%", height.max(2));
                                    html! {
                                        <div class="history-bar" style={height_style} title={format!("{} players", h.player_count)}></div>
                                    }
                                })}
                            </div>
                        </section>
                    }
                } else {
                    html! {}
                }}
                
                {if server.player_count > 0 {
                    html! {
                        <section class="details-section">
                            <h3>{"Online Players"}</h3>
                            <div class="player-list">
                                {for server.players.iter().map(|player| {
                                    html! { <span class="player-name">{player}</span> }
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
