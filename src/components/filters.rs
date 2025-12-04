use crate::utils::strip_all_tags;
use yew::prelude::*;

#[derive(Clone, PartialEq, Default)]
pub struct FilterState {
    pub search: String,
    pub version: String,
    pub has_players: bool,
    pub no_password: bool,
    pub tags: Vec<String>,
}

#[derive(Properties, PartialEq)]
pub struct FiltersProps {
    #[prop_or_default]
    pub current_search: String,
    #[prop_or_default]
    pub current_version: String,
    #[prop_or_default]
    pub has_players: bool,
    #[prop_or_default]
    pub no_password: bool,
    #[prop_or_default]
    pub versions: Vec<String>,
    #[prop_or_default]
    pub latest_version: String,
    #[prop_or_default]
    pub available_tags: Vec<String>,
    #[prop_or_default]
    pub selected_tags: Vec<String>,
}

/// Build URL with current filters, optionally toggling a tag
fn build_filter_url(props: &FiltersProps, toggle_tag: Option<&str>, clear_tags: bool) -> String {
    let mut params = Vec::new();
    
    if !props.current_search.is_empty() {
        params.push(format!("search={}", urlencoding::encode(&props.current_search)));
    }
    if !props.current_version.is_empty() {
        params.push(format!("version={}", urlencoding::encode(&props.current_version)));
    }
    if props.has_players {
        params.push("has_players=true".to_string());
    }
    if props.no_password {
        params.push("no_password=true".to_string());
    }
    
    // Handle tags
    if !clear_tags {
        let mut new_tags = props.selected_tags.clone();
        if let Some(tag) = toggle_tag {
            if let Some(pos) = new_tags.iter().position(|t| t == tag) {
                // Remove tag if already selected
                new_tags.remove(pos);
            } else {
                // Add tag if not selected
                new_tags.push(tag.to_string());
            }
        }
        if !new_tags.is_empty() {
            params.push(format!("tags={}", urlencoding::encode(&new_tags.join(","))));
        }
    }
    
    if params.is_empty() {
        "/".to_string()
    } else {
        format!("/?{}", params.join("&"))
    }
}

/// Filter controls component - renders as a form for SSR
/// In SSR mode, filters work via form submission / URL parameters
#[function_component(Filters)]
pub fn filters(props: &FiltersProps) -> Html {
    let is_latest_selected = props.current_version.is_empty();
    let is_all_selected = props.current_version == "all";
    
    // Create comma-separated string of selected tags for hidden input
    let selected_tags_value = props.selected_tags.join(",");
    let has_selected_tags = !props.selected_tags.is_empty();
    
    // Build URL for clearing all tags
    let clear_tags_url = build_filter_url(props, None, true);

    html! {
        <form id="filter-form" class="flex flex-col gap-4 mb-8 p-6 bg-bg-card/85 backdrop-blur-[10px] border border-border-subtle rounded-md" method="get" action="/">
            // Main filter controls row
            <div class="flex flex-wrap items-end gap-4">
                <div class="flex flex-col gap-1 flex-1 min-w-[200px]">
                    <label for="search" class="text-xs text-text-secondary uppercase tracking-wider">{"Search"}</label>
                    <input 
                        type="text" 
                        id="search"
                        name="search"
                        placeholder="Search servers..."
                        value={props.current_search.clone()}
                        class="py-2 px-4 bg-bg-inset border border-border-subtle rounded-sm text-text-primary font-display text-[0.95rem] transition-colors duration-200 focus:outline-none focus:border-accent-primary"
                    />
                </div>
                
                <div class="flex flex-col gap-1">
                    <label for="version" class="text-xs text-text-secondary uppercase tracking-wider">{"Version"}</label>
                    <select id="version" name="version" class="py-2 px-4 bg-bg-inset border border-border-subtle rounded-sm text-text-primary font-display text-[0.95rem] transition-colors duration-200 focus:outline-none focus:border-accent-primary">
                        <option value="" selected={is_latest_selected}>
                            {format!("Latest ({})", props.latest_version)}
                        </option>
                        <option value="all" selected={is_all_selected}>{"All Versions"}</option>
                        {for props.versions.iter().filter(|v| *v != &props.latest_version).map(|v| {
                            html! {
                                <option value={v.clone()} selected={&props.current_version == v}>
                                    {v}
                                </option>
                            }
                        })}
                    </select>
                </div>
                
                <div class="flex flex-col gap-1 justify-end">
                    <label class="flex items-center gap-2 cursor-pointer py-2 px-4 bg-bg-inset border border-border-subtle rounded-sm transition-colors duration-200 hover:border-accent-primary">
                        <input 
                            type="checkbox" 
                            name="has_players"
                            value="true"
                            checked={props.has_players}
                            class="accent-accent-primary w-4 h-4"
                        />
                        <span class="text-sm text-text-primary">{"Has Players"}</span>
                    </label>
                </div>
                
                <div class="flex flex-col gap-1 justify-end">
                    <label class="flex items-center gap-2 cursor-pointer py-2 px-4 bg-bg-inset border border-border-subtle rounded-sm transition-colors duration-200 hover:border-accent-primary">
                        <input 
                            type="checkbox" 
                            name="no_password"
                            value="true"
                            checked={props.no_password}
                            class="accent-accent-primary w-4 h-4"
                        />
                        <span class="text-sm text-text-primary">{"No Password"}</span>
                    </label>
                </div>
                
                <div class="flex flex-col gap-1 justify-end">
                    <button type="submit" class="py-2 px-6 bg-btn-green border border-btn-green-dark rounded-sm text-bg-dark font-display text-[0.95rem] font-semibold cursor-pointer transition-all duration-200 hover:bg-btn-green-hover active:bg-btn-green-dark">
                        {"Apply Filters"}
                    </button>
                </div>
            </div>
            
            // Tag pills row
            {if !props.available_tags.is_empty() {
                html! {
                    <div class="flex flex-col gap-2">
                        <div class="flex items-center gap-2">
                            <span class="text-xs text-text-secondary uppercase tracking-wider">{"Tags"}</span>
                            {if has_selected_tags {
                                html! {
                                    <a 
                                        href={clear_tags_url}
                                        class="text-xs text-accent-primary hover:text-accent-secondary transition-colors cursor-pointer no-underline"
                                    >
                                        {"Clear all"}
                                    </a>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                        <div class="flex flex-wrap gap-1 overflow-x-auto pb-1">
                            {for props.available_tags.iter().map(|tag| {
                                let is_selected = props.selected_tags.contains(tag);
                                let tag_escaped = strip_all_tags(tag);
                                let toggle_url = build_filter_url(props, Some(tag), false);
                                
                                // Match server card tag styling: py-1 px-2 bg-accent-glow border border-accent-primary rounded-sm text-xs text-accent-primary
                                let class = if is_selected {
                                    "py-1 px-2 bg-accent-primary border border-accent-primary rounded-sm text-xs text-bg-dark font-medium cursor-pointer transition-all duration-200 no-underline"
                                } else {
                                    "py-1 px-2 bg-accent-glow border border-accent-primary rounded-sm text-xs text-accent-primary cursor-pointer transition-all duration-200 no-underline hover:bg-accent-primary hover:text-bg-dark"
                                };
                                
                                html! {
                                    <a 
                                        href={toggle_url}
                                        class={class}
                                    >
                                        {tag_escaped}
                                    </a>
                                }
                            })}
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}
            
            // Hidden input for tags (used when form is submitted via Apply button)
            <input type="hidden" id="tags-input" name="tags" value={selected_tags_value} />
        </form>
    }
}
