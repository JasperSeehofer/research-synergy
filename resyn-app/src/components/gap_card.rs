use leptos::prelude::*;
use resyn_core::datamodels::gap_finding::{GapFinding, GapType};

use crate::app::SelectedPaper;

/// A card displaying a single gap finding with type badge, confidence bar,
/// shared terms, expandable justification, and clickable paper ID links.
#[component]
pub fn GapCard(finding: GapFinding) -> impl IntoView {
    let expanded = RwSignal::new(false);
    let selected_paper = use_context::<SelectedPaper>().expect("SelectedPaper context missing");

    let (badge_class, badge_label) = match finding.gap_type {
        GapType::Contradiction => ("badge badge-contradiction", "Contradiction"),
        GapType::AbcBridge => ("badge badge-bridge", "ABC-Bridge"),
    };

    // Confidence as percentage (0–100)
    let confidence_pct = (finding.confidence * 100.0).round() as u32;
    // Width style for the confidence bar fill
    let bar_width = format!("{}%", confidence_pct);

    let paper_ids = finding.paper_ids.clone();
    let shared_terms = finding.shared_terms.clone();
    let justification = finding.justification.clone();

    view! {
        <div class="gap-card">
            // Header: badge + found_at date
            <div class="gap-card-header">
                <span class=badge_class>{badge_label}</span>
                <span class="text-label text-muted">{finding.found_at.clone()}</span>
            </div>

            // Clickable paper IDs
            <div class="gap-card-titles">
                <For
                    each=move || paper_ids.clone()
                    key=|id| id.clone()
                    children=move |paper_id| {
                        let id_clone = paper_id.clone();
                        let set_paper = selected_paper.0;
                        view! {
                            <button
                                class="gap-card-paper-link"
                                on:click=move |_| set_paper.set(Some(id_clone.clone()))
                            >
                                {paper_id.clone()}
                            </button>
                        }
                    }
                />
            </div>

            // Shared terms as tag pills
            <div class="gap-card-shared-terms">
                <For
                    each=move || shared_terms.clone()
                    key=|t| t.clone()
                    children=move |term| {
                        view! { <span class="tag">{term}</span> }
                    }
                />
            </div>

            // Confidence bar
            <div class="gap-card-confidence">
                <div class="confidence-bar-wrapper">
                    <div class="confidence-bar">
                        <div
                            class="confidence-bar-fill"
                            style=format!("width: {bar_width}")
                        ></div>
                    </div>
                    <span class="confidence-label">{format!("{}%", confidence_pct)}</span>
                </div>
            </div>

            // Expand/collapse justification
            <button
                class="gap-card-expand-btn"
                on:click=move |_| expanded.update(|v| *v = !*v)
            >
                {move || if expanded.get() { "▲ Hide justification" } else { "▼ Show justification" }}
            </button>
            <div class=move || {
                if expanded.get() {
                    "gap-card-justification expanded"
                } else {
                    "gap-card-justification"
                }
            }>
                <p class="gap-card-justification-text">{justification.clone()}</p>
            </div>
        </div>
    }
}
