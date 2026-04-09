use crate::graph::layout_state::{ForceMode, LabelMode, SizeMode};
use leptos::prelude::*;

#[component]
pub fn GraphControls(
    show_contradictions: RwSignal<bool>,
    show_bridges: RwSignal<bool>,
    show_similarity: RwSignal<bool>,
    show_citations: RwSignal<bool>,
    force_mode: RwSignal<ForceMode>,
    simulation_running: RwSignal<bool>,
    zoom_in_count: RwSignal<u32>,
    zoom_out_count: RwSignal<u32>,
    visible_count: RwSignal<(usize, usize)>,
    temporal_min: RwSignal<u32>,
    temporal_max: RwSignal<u32>,
    year_bounds: RwSignal<(u32, u32)>,
    fit_count: RwSignal<u32>,
    simulation_settled: RwSignal<bool>,
    label_mode: RwSignal<LabelMode>,
    show_topic_rings: RwSignal<bool>,
    active_topic_filter: RwSignal<std::collections::HashSet<String>>,
    palette: RwSignal<Vec<crate::server_fns::graph::PaletteEntry>>,
    size_mode: RwSignal<SizeMode>,
    metrics_ready: RwSignal<bool>,
    metrics_computing: RwSignal<bool>,
) -> impl IntoView {
    let _ = temporal_min;
    let _ = temporal_max;
    let _ = year_bounds;
    view! {
        <div class="graph-controls-overlay">
            // Edge filter group
            <div class="graph-controls-group">
                <button
                    class=move || if show_citations.get() { "graph-control-btn active" } else { "graph-control-btn" }
                    on:click=move |_| show_citations.update(|v| *v = !*v)
                    aria-pressed=move || show_citations.get().to_string()
                    aria-label="Toggle citation edges"
                >
                    "Citations"
                </button>
                <button
                    class=move || if show_similarity.get() { "graph-control-btn active" } else { "graph-control-btn" }
                    on:click=move |_| show_similarity.update(|v| *v = !*v)
                    aria-pressed=move || show_similarity.get().to_string()
                    aria-label="Toggle similarity edges"
                >
                    "Similarity"
                </button>
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

            // Force mode selector (D-11: two distinct force models)
            <div class="graph-controls-group force-mode-selector">
                <span class="text-label" style="font-size: 12px; color: var(--color-text-muted); text-transform: uppercase;">"Layout"</span>
                <button
                    class=move || if force_mode.get() == ForceMode::Citation { "graph-control-btn active" } else { "graph-control-btn" }
                    on:click=move |_| force_mode.set(ForceMode::Citation)
                    aria-pressed=move || (force_mode.get() == ForceMode::Citation).to_string()
                    aria-label="Citation-driven layout"
                >
                    "Citation"
                </button>
                <button
                    class=move || if force_mode.get() == ForceMode::Similarity { "graph-control-btn active" } else { "graph-control-btn" }
                    on:click=move |_| force_mode.set(ForceMode::Similarity)
                    aria-pressed=move || (force_mode.get() == ForceMode::Similarity).to_string()
                    aria-label="Similarity-driven layout"
                >
                    "Similarity"
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
                    on:click=move |_| zoom_in_count.update(|v| *v = v.wrapping_add(1))
                    aria-label="Zoom in"
                >
                    "+"
                </button>
                <button
                    class="graph-control-btn"
                    on:click=move |_| zoom_out_count.update(|v| *v = v.wrapping_add(1))
                    aria-label="Zoom out"
                >
                    "\u{2212}"
                </button>
                <button
                    class="graph-control-btn"
                    on:click=move |_| fit_count.update(|v| *v = v.wrapping_add(1))
                    aria-label="Fit graph to viewport"
                >
                    "\u{2922}"
                </button>
            </div>

            // Node count indicator group
            <div class="graph-controls-group">
                <span
                    class=move || {
                        if simulation_running.get() { "sim-status-badge sim-running" }
                        else if simulation_settled.get() { "sim-status-badge sim-settled" }
                        else { "sim-status-badge sim-paused" }
                    }
                    aria-label=move || {
                        let state = if simulation_running.get() { "Simulating" }
                            else if simulation_settled.get() { "Settled" }
                            else { "Paused" };
                        format!("Simulation status: {}", state)
                    }
                >
                    {move || {
                        if simulation_running.get() { "Simulating..." }
                        else if simulation_settled.get() { "Settled" }
                        else { "Paused" }
                    }}
                </span>
                <span class="node-count-indicator">
                    {move || {
                        let (v, t) = visible_count.get();
                        format!("Showing {} of {} nodes", v, t)
                    }}
                </span>
            </div>

            // Label mode group (D-01)
            <div class="graph-controls-group">
                <span class="text-label" style="font-size: 12px; color: var(--color-text-muted); text-transform: uppercase;">"Label mode"</span>
                <select
                    class="form-select"
                    style="min-width: 120px;"
                    on:change=move |e| {
                        use leptos::wasm_bindgen::JsCast;
                        let val = e.target().unwrap()
                            .dyn_into::<web_sys::HtmlSelectElement>().unwrap()
                            .value();
                        label_mode.set(match val.as_str() {
                            "keywords" => LabelMode::Keywords,
                            "off" => LabelMode::Off,
                            _ => LabelMode::AuthorYear,
                        });
                    }
                >
                    <option value="author_year" selected=move || label_mode.get() == LabelMode::AuthorYear>"Author / Year"</option>
                    <option value="keywords" selected=move || label_mode.get() == LabelMode::Keywords>"Keywords"</option>
                    <option value="off" selected=move || label_mode.get() == LabelMode::Off>"Off"</option>
                </select>
            </div>

            // Topic ring toggle group (D-09)
            <div class="graph-controls-group">
                <button
                    class=move || if show_topic_rings.get() { "graph-control-btn active" } else { "graph-control-btn" }
                    on:click=move |_| show_topic_rings.update(|v| *v = !*v)
                    aria-pressed=move || show_topic_rings.get().to_string()
                    aria-label="Toggle topic ring borders"
                >
                    "Topic Rings"
                </button>
            </div>

            // Size by group (D-01, D-08, D-10)
            <div class="graph-controls-group">
                <span class="text-label" style="font-size: 12px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase;">
                    {move || if metrics_computing.get() {
                        view! { "Size by " <span class="spinner-sm"></span> }.into_any()
                    } else {
                        view! { "Size by" }.into_any()
                    }}
                </span>
                <select
                    class="form-select"
                    style="min-width: 120px;"
                    on:change=move |e| {
                        use leptos::wasm_bindgen::JsCast;
                        let val = e.target().unwrap()
                            .dyn_into::<web_sys::HtmlSelectElement>().unwrap()
                            .value();
                        size_mode.set(match val.as_str() {
                            "pagerank" => SizeMode::PageRank,
                            "betweenness" => SizeMode::Betweenness,
                            "citations" => SizeMode::Citations,
                            _ => SizeMode::Uniform,
                        });
                    }
                >
                    <option value="uniform" selected=move || size_mode.get() == SizeMode::Uniform>"Uniform"</option>
                    <option value="pagerank"
                        prop:disabled=move || !metrics_ready.get()>
                        {move || if !metrics_ready.get() {
                            "PageRank (computing\u{2026})"
                        } else {
                            "PageRank"
                        }}
                    </option>
                    <option value="betweenness"
                        prop:disabled=move || !metrics_ready.get()>
                        {move || if !metrics_ready.get() {
                            "Betweenness (computing\u{2026})"
                        } else {
                            "Betweenness"
                        }}
                    </option>
                    <option value="citations" selected=move || size_mode.get() == SizeMode::Citations>"Citations"</option>
                </select>
                <button
                    class="graph-control-btn"
                    title="Recompute centrality metrics"
                    aria-label="Recompute graph metrics"
                    prop:disabled=move || metrics_computing.get()
                    on:click=move |_| {
                        use crate::server_fns::metrics::trigger_metrics_compute;
                        leptos::task::spawn_local(async move {
                            let _ = trigger_metrics_compute().await;
                        });
                    }
                >
                    {move || if metrics_computing.get() {
                        view! { <span class="spinner-sm"></span> }.into_any()
                    } else {
                        view! { "\u{21BA}" }.into_any()
                    }}
                </button>
            </div>

            // Topic Colors legend (D-10, D-11, D-12)
            {move || {
                let rings_on = show_topic_rings.get();
                let pal = palette.get();
                if rings_on && !pal.is_empty() {
                    Some(view! {
                        <div class="topic-legend-section">
                            <div class="sidebar-title">"TOPIC COLORS"</div>
                            <div class="topic-legend-entries">
                                {pal.into_iter().map(|entry| {
                                    let kw = entry.keyword.clone();
                                    let kw_click = kw.clone();
                                    let kw_class = kw.clone();
                                    let swatch_style = format!(
                                        "background: rgb({},{},{});",
                                        entry.r, entry.g, entry.b
                                    );
                                    view! {
                                        <button
                                            class=move || {
                                                let active = active_topic_filter.get().contains(&kw_class);
                                                if active { "legend-entry active" } else { "legend-entry" }
                                            }
                                            on:click=move |_| {
                                                active_topic_filter.update(|set| {
                                                    let k = kw_click.clone();
                                                    if set.contains(&k) {
                                                        set.remove(&k);
                                                    } else {
                                                        set.insert(k);
                                                    }
                                                });
                                            }
                                        >
                                            <span class="legend-swatch" style=swatch_style.clone()></span>
                                            {kw.clone()}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}

#[component]
pub fn TemporalSlider(
    temporal_min: RwSignal<u32>,
    temporal_max: RwSignal<u32>,
    year_bounds: RwSignal<(u32, u32)>,
) -> impl IntoView {
    view! {
        <div class="temporal-slider-row">
            <label class="text-label">"Year range"</label>
            <div class="dual-range-wrapper">
                <input
                    type="range"
                    class="temporal-range temporal-range-min"
                    min=move || year_bounds.get().0
                    max=move || year_bounds.get().1
                    prop:value=move || temporal_min.get()
                    on:input=move |e| {
                        use leptos::wasm_bindgen::JsCast;
                        let val = e.target().unwrap()
                            .dyn_into::<web_sys::HtmlInputElement>().unwrap()
                            .value_as_number() as u32;
                        temporal_min.set(val.min(temporal_max.get_untracked()));
                    }
                />
                <input
                    type="range"
                    class="temporal-range temporal-range-max"
                    min=move || year_bounds.get().0
                    max=move || year_bounds.get().1
                    prop:value=move || temporal_max.get()
                    on:input=move |e| {
                        use leptos::wasm_bindgen::JsCast;
                        let val = e.target().unwrap()
                            .dyn_into::<web_sys::HtmlInputElement>().unwrap()
                            .value_as_number() as u32;
                        temporal_max.set(val.max(temporal_min.get_untracked()));
                    }
                />
            </div>
            <span class="text-label">
                {move || format!("{} \u{2013} {}", temporal_min.get(), temporal_max.get())}
            </span>
        </div>
    }
}
