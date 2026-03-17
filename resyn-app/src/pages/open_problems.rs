use leptos::prelude::*;
use resyn_core::analysis::aggregation::RankedProblem;

use crate::server_fns::problems::get_open_problems_ranked;

/// Open Problems panel — ranked list sorted by recurrence frequency.
#[component]
pub fn OpenProblemsPanel() -> impl IntoView {
    let problems = Resource::new(|| (), |_| get_open_problems_ranked());

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
                    Ok(items) if items.is_empty() => view! {
                        <div class="empty-state">
                            <p class="empty-state-body">
                                "No open problems extracted. Run LLM analysis on the crawled papers."
                            </p>
                        </div>
                    }.into_any(),
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
    let ranked: Vec<(usize, RankedProblem)> =
        items.into_iter().enumerate().map(|(i, p)| (i + 1, p)).collect();

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
