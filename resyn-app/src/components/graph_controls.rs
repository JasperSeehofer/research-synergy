use leptos::prelude::*;

#[component]
pub fn GraphControls(
    show_contradictions: RwSignal<bool>,
    show_bridges: RwSignal<bool>,
    simulation_running: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="graph-controls-overlay">
            // Edge filter group
            <div class="graph-controls-group">
                <button
                    class=move || if show_contradictions.get() { "graph-control-btn active" } else { "graph-control-btn" }
                    on:click=move |_| show_contradictions.update(|v| *v = !*v)
                    aria-pressed=move || show_contradictions.get().to_string()
                    aria-label="Toggle contradiction edges"
                >
                    "Contradiction"
                </button>
                <button
                    class=move || if show_bridges.get() { "graph-control-btn active" } else { "graph-control-btn" }
                    on:click=move |_| show_bridges.update(|v| *v = !*v)
                    aria-pressed=move || show_bridges.get().to_string()
                    aria-label="Toggle ABC-Bridge edges"
                >
                    "ABC-Bridge"
                </button>
            </div>

            // Simulation control group
            <div class="graph-controls-group">
                <button
                    class="graph-control-btn"
                    on:click=move |_| simulation_running.update(|v| *v = !*v)
                    aria-pressed=move || simulation_running.get().to_string()
                    aria-label=move || if simulation_running.get() { "Pause force simulation" } else { "Resume force simulation" }
                >
                    {move || if simulation_running.get() { "\u{23F8}" } else { "\u{25B6}" }}
                </button>
                <button
                    class="graph-control-btn"
                    aria-label="Zoom in"
                    data-action="zoom-in"
                >
                    "+"
                </button>
                <button
                    class="graph-control-btn"
                    aria-label="Zoom out"
                    data-action="zoom-out"
                >
                    "\u{2212}"
                </button>
            </div>
        </div>
    }
}
