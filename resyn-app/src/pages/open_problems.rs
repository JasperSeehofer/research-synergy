use leptos::prelude::*;

/// Open Problems panel — ranked list by recurrence frequency.
#[component]
pub fn OpenProblemsPanel() -> impl IntoView {
    view! {
        <div>
            <h1 class="page-title">"Open Problems"</h1>
            <div class="empty-state">
                <p class="empty-state-body">"No open problems extracted. Run LLM analysis on the crawled papers."</p>
            </div>
        </div>
    }
}
