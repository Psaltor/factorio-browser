use yew::prelude::*;

#[derive(Clone, PartialEq, Default)]
pub struct FilterState {
    pub search: String,
    pub version: String,
    pub has_players: bool,
    pub no_password: bool,
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
}

/// Filter controls component - renders as a form for SSR
/// In SSR mode, filters work via form submission / URL parameters
#[function_component(Filters)]
pub fn filters(props: &FiltersProps) -> Html {
    let is_latest_selected = props.current_version.is_empty();
    let is_all_selected = props.current_version == "all";

    html! {
        <form class="flex flex-wrap items-end gap-4 mb-8 p-6 bg-bg-card/85 backdrop-blur-[10px] border border-border-subtle rounded-md" method="get" action="/">
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
        </form>
    }
}
