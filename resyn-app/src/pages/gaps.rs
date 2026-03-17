use leptos::prelude::*;

/// Gaps panel — gap findings with filter controls.
#[component]
pub fn GapsPanel() -> impl IntoView {
    let show_contradictions = RwSignal::new(true);
    let show_bridges = RwSignal::new(true);
    let min_confidence = RwSignal::new(0u8);

    view! {
        <div>
            <h1 class="page-title">"Gaps"</h1>
            <div class="filter-bar">
                <button
                    class=move || if show_contradictions.get() { "filter-toggle active" } else { "filter-toggle" }
                    on:click=move |_| show_contradictions.update(|v| *v = !*v)
                >
                    "Contradictions"
                </button>
                <button
                    class=move || if show_bridges.get() { "filter-toggle active" } else { "filter-toggle" }
                    on:click=move |_| show_bridges.update(|v| *v = !*v)
                >
                    "Bridges"
                </button>
                <div class="slider-wrapper">
                    <label class="slider-label">
                        "Min confidence: "
                        {move || format!("{}%", min_confidence.get())}
                    </label>
                    <input
                        type="range"
                        min="0"
                        max="100"
                        step="5"
                        prop:value=move || min_confidence.get().to_string()
                        on:input=move |e| {
                            if let Ok(v) = event_target_value(&e).parse::<u8>() {
                                min_confidence.set(v);
                            }
                        }
                    />
                </div>
            </div>
            <div class="empty-state">
                <p class="empty-state-body">"No gap findings yet. Run analysis after crawling papers."</p>
            </div>
        </div>
    }
}
