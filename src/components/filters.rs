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
}

/// Filter controls component - renders as a form for SSR
/// In SSR mode, filters work via form submission / URL parameters
#[function_component(Filters)]
pub fn filters(props: &FiltersProps) -> Html {
    html! {
        <form class="filters" method="get" action="/">
            <div class="filter-group search-group">
                <label for="search">{"Search"}</label>
                <input 
                    type="text" 
                    id="search"
                    name="search"
                    placeholder="Search servers..."
                    value={props.current_search.clone()}
                />
            </div>
            
            <div class="filter-group">
                <label for="version">{"Version"}</label>
                <select id="version" name="version">
                    <option value="" selected={props.current_version.is_empty()}>{"All Versions"}</option>
                    {for props.versions.iter().map(|v| {
                        html! {
                            <option value={v.clone()} selected={&props.current_version == v}>
                                {v}
                            </option>
                        }
                    })}
                </select>
            </div>
            
            <div class="filter-group checkbox-group">
                <label>
                    <input 
                        type="checkbox" 
                        name="has_players"
                        value="true"
                        checked={props.has_players}
                    />
                    <span>{"Has Players"}</span>
                </label>
            </div>
            
            <div class="filter-group checkbox-group">
                <label>
                    <input 
                        type="checkbox" 
                        name="no_password"
                        value="true"
                        checked={props.no_password}
                    />
                    <span>{"No Password"}</span>
                </label>
            </div>
            
            <div class="filter-group">
                <button type="submit" class="filter-button">{"Apply Filters"}</button>
            </div>
        </form>
    }
}
