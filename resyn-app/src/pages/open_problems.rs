use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::{UseEventSourceReturn, use_event_source};
use resyn_core::analysis::aggregation::RankedProblem;
use resyn_core::datamodels::progress::ProgressEvent;

use crate::server_fns::problems::get_open_problems_ranked;

/// Open Problems panel — ranked list sorted by recurrence frequency.
#[component]
pub fn OpenProblemsPanel() -> impl IntoView {
    let problems = Resource::new(|| (), |_| get_open_problems_ranked());

    let UseEventSourceReturn {
        message: sse_message,
        ..
    } = use_event_source::<ProgressEvent, JsonSerdeCodec>("/progress");

    Effect::new(move |_| {
        if let Some(msg) = sse_message.get() {
            if msg.data.event_type == "analysis_complete" {
                problems.refetch();
            }
        }
    });

    view! {
        <div>
            <h1 class="page-title">"Open Problems"</h1>

            <Suspense fallback=|| view! {
                <div class="ranked-list">
                    <div class="skeleton skeleton-row" style="margin-bottom: 1px;"></div>
                    <div class="skeleton skeleton-row" style="margin-bottom: 1px;"></div>
                    <div class="skeleton skeleton-row" style="margin-bottom: 1px;"></div>
                    <div class="skeleton skeleton-row" style="margin-bottom: 1px;"></div>
                    <div class="skeleton skeleton-row"></div>
                </div>
            }>
                {move || problems.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="error-banner">
                            {format!(
                                "Failed to load open problems. Check the server connection and retry. ({e})"
                            )}
                        </div>
                    }.into_any(),
                    Ok(items) if items.is_empty() => {
                        let analysis_action = Action::new(move |_: &()| async move {
                            crate::server_fns::analysis::start_analysis().await
                        });
                        view! {
                            <div class="empty-state">
                                <p class="empty-state-heading">"No analysis results yet"</p>
                                <p class="empty-state-body">
                                    "Run analysis to see open problems here."
                                </p>
                                <button
                                    class="btn-primary"
                                    on:click=move |_| { analysis_action.dispatch(()); }
                                >
                                    "Run Analysis"
                                </button>
                            </div>
                        }.into_any()
                    },
                    Ok(items) => view! {
                        <RankedList items=items/>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

/// A numbered ranked list of open problems with recurrence count badges.
#[component]
fn RankedList(items: Vec<RankedProblem>) -> impl IntoView {
    // Assign rank numbers (1-based) before moving into the reactive closure.
    let ranked: Vec<(usize, RankedProblem)> = items
        .into_iter()
        .enumerate()
        .map(|(i, p)| (i + 1, p))
        .collect();

    view! {
        <ul class="ranked-list" style="list-style: none; padding: 0; margin: 0;">
            <For
                each=move || ranked.clone()
                key=|(rank, p)| format!("{}-{}", rank, p.problem)
                children=move |(rank, problem)| {
                    view! {
                        <li class="ranked-item">
                            <span class="rank-number">
                                {format!("#{rank}")}
                            </span>
                            <span class="ranked-problem-text">
                                {problem.problem.clone()}
                            </span>
                            <span class="recurrence-badge">
                                {problem.count.to_string()}
                            </span>
                        </li>
                    }
                }
            />
        </ul>
    }
}
