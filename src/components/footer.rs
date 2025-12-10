use chrono::Datelike;
use yew::prelude::*;

/// Reusable footer component
#[function_component(Footer)]
pub fn footer() -> Html {
    let current_year = chrono::Utc::now().year();

    html! {
        <footer class="text-center p-6 text-text-muted text-sm">
            <p>{format!("© {} • Source code available at ", current_year)}<a href="https://github.com/Psaltor/factorio-browser" target="_blank" class="text-accent-primary hover:text-accent-secondary transition-colors" target="_blank" rel="noopener">{"Github.com"}</a></p>
            <p class="mt-1">{"Data from Factorio Matchmaking API • Not affiliated with Wube Software"}</p>
        </footer>
    }
}

