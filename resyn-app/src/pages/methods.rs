use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::{UseEventSourceReturn, use_event_source};
use resyn_core::datamodels::progress::ProgressEvent;

use crate::components::heatmap::Heatmap;
use crate::server_fns::methods::{get_method_drilldown, get_method_matrix};

/// Methods panel — method-combination heatmap with drill-down.
#[component]
pub fn MethodsPanel() -> impl IntoView {
    // Primary matrix resource.
    let matrix_resource = Resource::new(|| (), |_| async { get_method_matrix().await });

    let UseEventSourceReturn { message: sse_message, .. } =
        use_event_source::<ProgressEvent, JsonSerdeCodec>("/progress");

    Effect::new(move |_| {
        if let Some(msg) = sse_message.get() {
            if msg.data.event_type == "analysis_complete" {
                matrix_resource.refetch();
            }
        }
    });

    let analysis_action = Action::new(move |_: &()| async move {
        crate::server_fns::analysis::start_analysis().await
    });

    // Drill-down state: None = overview, Some((row_cat, col_cat)) = drill-down view.
    let drilldown: RwSignal<Option<(String, String)>> = RwSignal::new(None);

    // Drilldown matrix resource — only fetches when drilldown is Some.
    let drilldown_resource = Resource::new(
        move || drilldown.get(),
        |pair| async move {
            match pair {
                Some((cat_a, cat_b)) => Some(get_method_drilldown(cat_a, cat_b).await),
                None => None,
            }
        },
    );

    view! {
        <div>
            <h1 class="page-title">"Methods"</h1>
            <p class="text-body text-muted" style="margin-bottom: var(--space-lg);">
                "Method co-occurrence matrix across all annotated papers. Click a cell to drill down into individual method names."
            </p>

            {move || {
                // If drill-down is active, show the sub-matrix.
                if let Some((cat_a, cat_b)) = drilldown.get() {
                    let label = format!("{} × {}", cat_a, cat_b);
                    view! {
                        <div>
                            <div style="display:flex;align-items:center;gap:var(--space-md);margin-bottom:var(--space-lg);">
                                <button
                                    class="btn-secondary"
                                    on:click=move |_| drilldown.set(None)
                                >
                                    "← Back to overview"
                                </button>
                                <h2 class="text-heading">{label}</h2>
                            </div>
                            <Suspense fallback=|| view! {
                                <div class="spinner"></div>
                            }>
                                {move || {
                                    drilldown_resource.get().map(|result| {
                                        match result {
                                            Some(Ok(matrix)) => {
                                                if matrix.categories.is_empty() {
                                                    view! {
                                                        <div class="empty-state">
                                                            <p class="empty-state-body">"No individual method data for this category pair."</p>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="heatmap-container">
                                                            <Heatmap matrix=matrix/>
                                                        </div>
                                                    }.into_any()
                                                }
                                            }
                                            Some(Err(e)) => view! {
                                                <div class="error-banner">{format!("Error loading drill-down: {}", e)}</div>
                                            }.into_any(),
                                            None => view! { <div class="spinner"></div> }.into_any(),
                                        }
                                    })
                                }}
                            </Suspense>
                        </div>
                    }.into_any()
                } else {
                    // Overview heatmap.
                    view! {
                        <Suspense fallback=|| view! {
                            <div style="display:flex;align-items:center;gap:var(--space-sm);">
                                <div class="spinner spinner-sm"></div>
                                <span class="text-muted">"Loading method matrix…"</span>
                            </div>
                        }>
                            {move || {
                                matrix_resource.get().map(|result| {
                                    match result {
                                        Ok(matrix) => {
                                            if matrix.categories.is_empty() {
                                                view! {
                                                    <div class="empty-state">
                                                        <p class="empty-state-heading">"No analysis results yet"</p>
                                                        <p class="empty-state-body">"Run analysis to see the method heatmap here."</p>
                                                        <button
                                                            class="btn-primary"
                                                            on:click=move |_| { analysis_action.dispatch(()); }
                                                        >
                                                            "Run Analysis"
                                                        </button>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                let on_click = Callback::new(move |(row, col): (String, String)| {
                                                    drilldown.set(Some((row, col)));
                                                });
                                                view! {
                                                    <div class="heatmap-container">
                                                        <Heatmap matrix=matrix on_cell_click=on_click/>
                                                    </div>
                                                }.into_any()
                                            }
                                        }
                                        Err(e) => view! {
                                            <div class="error-banner">{format!("Error loading method matrix: {}", e)}</div>
                                        }.into_any(),
                                    }
                                })
                            }}
                        </Suspense>
                    }.into_any()
                }
            }}
        </div>
    }
}
