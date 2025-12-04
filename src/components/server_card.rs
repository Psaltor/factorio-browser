use crate::db::models::CachedServer;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ServerCardProps {
    pub server: CachedServer,
}

/// Individual server card component (SSR-compatible)
#[function_component(ServerCard)]
pub fn server_card(props: &ServerCardProps) -> Html {
    let server = &props.server;
    let player_ratio = if server.max_players > 0 {
        (server.player_count as f32 / server.max_players as f32 * 100.0) as u32
    } else {
        0
    };

    let player_class = if player_ratio >= 80 {
        "player-count high"
    } else if player_ratio >= 50 {
        "player-count medium"
    } else if server.player_count > 0 {
        "player-count low"
    } else {
        "player-count empty"
    };

    // Format game time (API returns minutes)
    let hours = server.game_time_elapsed / 60;
    let minutes = server.game_time_elapsed % 60;
    let game_time = format!("{}h {}m", hours, minutes);

    // Link to server details page
    let details_url = format!("/server/{}", server.game_id);

    html! {
        <a href={details_url} class="server-card">
            <div class="server-header">
                <h3 class="server-name">{&server.name}</h3>
                {if server.has_password {
                    html! { <span class="password-badge" title="Password Protected">{"🔒"}</span> }
                } else {
                    html! {}
                }}
            </div>
            
            <div class="server-info">
                <div class={player_class}>
                    <span class="player-icon">{"👥"}</span>
                    <span>{format!("{}/{}", server.player_count, server.max_players)}</span>
                </div>
                
                <div class="server-version">
                    <span class="version-icon">{"🎮"}</span>
                    <span>{&server.game_version}</span>
                </div>
                
                <div class="server-time">
                    <span class="time-icon">{"⏱️"}</span>
                    <span>{game_time}</span>
                </div>
                
                {if server.mod_count > 0 {
                    html! {
                        <div class="server-mods">
                            <span class="mods-icon">{"📦"}</span>
                            <span>{format!("{} mods", server.mod_count)}</span>
                        </div>
                    }
                } else {
                    html! {
                        <div class="server-mods vanilla">
                            <span>{"Vanilla"}</span>
                        </div>
                    }
                }}
            </div>
            
            {if !server.description.is_empty() {
                html! {
                    <p class="server-description">{&server.description}</p>
                }
            } else {
                html! {}
            }}
            
            {if !server.tags.is_empty() {
                html! {
                    <div class="server-tags">
                        {for server.tags.iter().take(5).map(|tag| {
                            html! { <span class="tag">{tag}</span> }
                        })}
                    </div>
                }
            } else {
                html! {}
            }}
        </a>
    }
}
