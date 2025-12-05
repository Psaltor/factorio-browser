use crate::db::models::CachedServer;
use crate::utils::parse_rich_text;
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

    let player_color_class = if player_ratio >= 80 {
        "text-status-full"
    } else if player_ratio >= 50 {
        "text-status-medium"
    } else if server.player_count > 0 {
        "text-status-low"
    } else {
        "text-status-empty"
    };

    // Format game time (API returns minutes)
    let hours = server.game_time_elapsed / 60;
    let minutes = server.game_time_elapsed % 60;
    let game_time = format!("{}h {}m", hours, minutes);

    // Link to server details page
    let details_url = format!("/server/{}", server.game_id);

    let mods_display = if server.mod_count > 0 {
        format!("{} mods", server.mod_count)
    } else {
        "Vanilla".to_string()
    };

    html! {
        <div class="server-item contents" data-players={server.player_count.to_string()} data-time={server.game_time_elapsed.to_string()}>
            // Card view
            <a href={details_url.clone()} class="server-card block no-underline text-inherit bg-bg-card border border-border-subtle rounded-md p-6 cursor-pointer transition-all duration-200 hover:border-accent-primary hover:bg-bg-elevated">
                <div class="flex items-start justify-between gap-2 mb-4">
                    <h3 class="text-lg font-normal leading-tight break-words">{parse_rich_text(&server.name)}</h3>
                    {if server.has_password {
                        html! { <span class="flex-shrink-0 text-base" title="Password Protected">{"🔒"}</span> }
                    } else {
                        html! {}
                    }}
                </div>
                
                <div class="flex flex-wrap gap-2 mb-4">
                    <div class={classes!("flex", "items-center", "gap-1", "py-1", "px-2", "bg-bg-dark", "rounded-sm", "text-[0.85rem]", "font-mono", player_color_class)}>
                        <span>{"👥"}</span>
                        <span>{format!("{}/{}", server.player_count, server.max_players)}</span>
                    </div>
                    
                    <div class="flex items-center gap-1 py-1 px-2 bg-bg-dark rounded-sm text-[0.85rem] font-mono">
                        <span>{"🎮"}</span>
                        <span>{&server.game_version}</span>
                    </div>
                    
                    <div class="flex items-center gap-1 py-1 px-2 bg-bg-dark rounded-sm text-[0.85rem] font-mono">
                        <span>{"⏱️"}</span>
                        <span>{&game_time}</span>
                    </div>
                    
                    {if server.mod_count > 0 {
                        html! {
                            <div class="flex items-center gap-1 py-1 px-2 bg-bg-dark rounded-sm text-[0.85rem] font-mono">
                                <span>{"📦"}</span>
                                <span>{format!("{} mods", server.mod_count)}</span>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="py-1 px-2 bg-bg-dark rounded-sm text-[0.85rem] font-mono text-text-muted italic">
                                <span>{"Vanilla"}</span>
                            </div>
                        }
                    }}
                </div>
                
                {if !server.description.is_empty() {
                    html! {
                        <p class="text-sm text-text-secondary mb-4 line-clamp-2">{parse_rich_text(&server.description)}</p>
                    }
                } else {
                    html! {}
                }}
                
                {if !server.tags.is_empty() {
                    html! {
                        <div class="flex flex-wrap gap-1">
                            {for server.tags.iter().take(5).map(|tag| {
                                html! { <span class="py-1 px-2 bg-accent-glow border border-accent-primary rounded-sm text-xs text-accent-primary">{parse_rich_text(tag)}</span> }
                            })}
                        </div>
                    }
                } else {
                    html! {}
                }}
            </a>
            
            // List row view
            <a href={details_url} class="server-row hidden items-center gap-4 py-2 px-4 bg-bg-card border border-border-subtle rounded-sm no-underline text-text-primary transition-all duration-200 hover:border-accent-primary hover:bg-bg-elevated">
                <span class="flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap font-medium">
                    {parse_rich_text(&server.name)}
                    {if server.has_password {
                        html! { <span class="ml-1 text-[0.85em]">{"🔒"}</span> }
                    } else {
                        html! {}
                    }}
                </span>
                <span class="w-[60px] text-center text-accent-secondary font-medium">{format!("{}/{}", server.player_count, server.max_players)}</span>
                <span class="w-[70px] text-center text-text-secondary text-sm">{&server.game_version}</span>
                <span class="w-[80px] text-center text-text-muted text-sm">{&game_time}</span>
                <span class="w-[80px] text-right text-text-muted text-[0.85rem]">{&mods_display}</span>
            </a>
        </div>
    }
}
