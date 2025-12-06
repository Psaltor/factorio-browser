use chrono::Datelike;
use yew::prelude::*;

/// Reusable footer component
#[function_component(Footer)]
pub fn footer() -> Html {
    let current_year = chrono::Utc::now().year();

    html! {
        <footer class="text-center p-6 text-text-muted text-sm">
            <p>{format!("© {} • Brought to you by ", current_year)}<a href="https://lambs.cafe" class="text-accent-primary hover:text-accent-secondary transition-colors" target="_blank" rel="noopener">{"lambs.cafe"}</a></p>
            <p class="mt-1">{"Data from Factorio Matchmaking API • Not affiliated with Wube Software"}</p>
        </footer>
    }
}

