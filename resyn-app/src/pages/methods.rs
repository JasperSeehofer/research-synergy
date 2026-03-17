use leptos::prelude::*;

/// Methods panel — method-combination heatmap.
#[component]
pub fn MethodsPanel() -> impl IntoView {
    view! {
        <div>
            <h1 class="page-title">"Methods"</h1>
            <p class="text-body text-muted" style="margin-bottom: var(--space-lg);">
                "Method co-occurrence matrix across all annotated papers. Click a cell to drill down into individual method names."
            </p>
            <div class="empty-state">
                <p class="empty-state-body">"No method annotations found. Run LLM analysis on the crawled papers."</p>
            </div>
        </div>
    }
}
