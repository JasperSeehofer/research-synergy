use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::{UseEventSourceReturn, use_event_source};
use resyn_core::datamodels::gap_finding::GapType;
use resyn_core::datamodels::progress::ProgressEvent;

use crate::components::gap_card::GapCard;
use crate::server_fns::gaps::get_gap_findings;

/// Gaps panel — gap findings with type filter toggles and confidence threshold.
#[component]
pub fn GapsPanel() -> impl IntoView {
    let findings = Resource::new(|| (), |_| get_gap_findings());

    let UseEventSourceReturn {
        message: sse_message,
        ..
    } = use_event_source::<ProgressEvent, JsonSerdeCodec>("/progress");

    Effect::new(move |_| {
        if let Some(msg) = sse_message.get() {
            if msg.data.event_type == "analysis_complete" {
                findings.refetch();
            }
        }
    });

    let show_contradictions = RwSignal::new(true);
    let show_bridges = RwSignal::new(true);
    let min_confidence = RwSignal::new(0u32);

    view! {
        <div>
            <h1 class="page-title">"Gap Findings"</h1>

            // Filter bar
            <div class="filter-bar">
                <button
                    class=move || {
                        if show_contradictions.get() {
                            "filter-toggle active"
                        } else {
                            "filter-toggle"
                        }
                    }
                    on:click=move |_| show_contradictions.update(|v| *v = !*v)
                >
                    "Contradictions"
                </button>
                <button
                    class=move || {
                        if show_bridges.get() {
                            "filter-toggle active"
                        } else {
                            "filter-toggle"
                        }
                    }
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
                            if let Ok(v) = event_target_value(&e).parse::<u32>() {
                                min_confidence.set(v);
                            }
                        }
                    />
                </div>
            </div>

            // Data panel
            <Suspense fallback=|| view! {
                <div>
                    <div class="skeleton skeleton-card" style="margin-bottom: var(--space-lg);"></div>
                    <div class="skeleton skeleton-card" style="margin-bottom: var(--space-lg);"></div>
                    <div class="skeleton skeleton-card" style="margin-bottom: var(--space-lg);"></div>
                </div>
            }>
                {move || findings.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="error-banner">
                            {format!("Failed to load gap findings. Check the server connection and retry. ({e})")}
                        </div>
                    }.into_any(),
                    Ok(all_findings) => {
                        // Wrap in a signal so derived filtering can react to filter changes
                        let all_findings = StoredValue::new(all_findings);

                        // Derived list reacts to filter signals
                        let filtered = move || {
                            let show_c = show_contradictions.get();
                            let show_b = show_bridges.get();
                            let min_conf = min_confidence.get();
                            all_findings
                                .get_value()
                                .into_iter()
                                .filter(|f| {
                                    let type_ok = match &f.gap_type {
                                        GapType::Contradiction => show_c,
                                        GapType::AbcBridge => show_b,
                                    };
                                    let conf_ok =
                                        (f.confidence * 100.0).round() as u32 >= min_conf;
                                    type_ok && conf_ok
                                })
                                .collect::<Vec<_>>()
                        };

                        view! {
                            {move || {
                                let list = filtered();
                                if all_findings.get_value().is_empty() {
                                    let analysis_action = Action::new(move |_: &()| async move {
                                        crate::server_fns::analysis::start_analysis().await
                                    });
                                    view! {
                                        <div class="empty-state">
                                            <p class="empty-state-heading">"No analysis results yet"</p>
                                            <p class="empty-state-body">
                                                "Run analysis to see gap findings here."
                                            </p>
                                            <button
                                                class="btn-primary"
                                                on:click=move |_| { analysis_action.dispatch(()); }
                                            >
                                                "Run Analysis"
                                            </button>
                                        </div>
                                    }.into_any()
                                } else if list.is_empty() {
                                    view! {
                                        <div class="empty-state">
                                            <p class="empty-state-body">
                                                "No findings match the current filters. Adjust the confidence threshold or toggle finding types."
                                            </p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div>
                                            <For
                                                each=move || list.clone()
                                                key=|f| {
                                                    format!(
                                                        "{}-{}",
                                                        f.found_at,
                                                        f.paper_ids.join(",")
                                                    )
                                                }
                                                children=move |finding| {
                                                    view! { <GapCard finding=finding/> }
                                                }
                                            />
                                        </div>
                                    }.into_any()
                                }
                            }}
                        }.into_any()
                    }
                })}
            </Suspense>
        </div>
    }
}
