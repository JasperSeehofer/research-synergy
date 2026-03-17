use leptos::prelude::*;
use leptos::web_sys;

use crate::app::SelectedPaper;
use crate::server_fns::papers::{PaperDetail, get_paper_detail};

/// Paper detail side drawer. Slides in from the right when a paper is selected.
///
/// Controlled by the `SelectedPaper` context signal. Setting to `None` closes the drawer.
#[component]
pub fn Drawer() -> impl IntoView {
    let SelectedPaper(selected) = expect_context::<SelectedPaper>();

    let drawer_class = move || {
        if selected.get().is_some() {
            "drawer open"
        } else {
            "drawer closed"
        }
    };

    let close = move |_: web_sys::MouseEvent| selected.set(None);

    view! {
        // Backdrop — click to close
        {move || selected.get().is_some().then(|| view! {
            <div
                class="drawer-backdrop"
                on:click=close
                aria-hidden="true"
            ></div>
        })}

        <aside
            class=drawer_class
            role="dialog"
            aria-modal="true"
            aria-label="Paper details"
        >
            {move || {
                if let Some(paper_id) = selected.get() {
                    view! {
                        <DrawerContent paper_id=paper_id on_close=close/>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </aside>
    }
}

/// Content rendered inside the open drawer for a given paper_id.
#[component]
fn DrawerContent(
    paper_id: String,
    #[prop(into)] on_close: Callback<web_sys::MouseEvent>,
) -> impl IntoView {
    let id = paper_id.clone();
    let detail_resource = Resource::new(move || id.clone(), get_paper_detail);

    view! {
        <Suspense fallback=move || view! {
            <DrawerSkeleton paper_id=paper_id.clone() on_close=on_close/>
        }>
            {move || detail_resource.get().map(|result| match result {
                Err(e) => view! {
                    <div class="drawer-header">
                        <h2 class="drawer-title">"Error loading paper"</h2>
                        <button
                            class="drawer-close"
                            on:click=move |e| on_close.run(e)
                            aria-label="Close"
                        >
                            "✕"
                        </button>
                    </div>
                    <div class="drawer-body">
                        <div class="error-banner">
                            {format!("Failed to load paper details: {e}")}
                        </div>
                    </div>
                }.into_any(),
                Ok(detail) => view! {
                    <DrawerBody detail=detail on_close=on_close/>
                }.into_any(),
            })}
        </Suspense>
    }
}

/// Drawer body once paper detail is loaded.
#[component]
fn DrawerBody(
    detail: PaperDetail,
    #[prop(into)] on_close: Callback<web_sys::MouseEvent>,
) -> impl IntoView {
    let paper = detail.paper;
    let annotation = detail.annotation;

    let year = if paper.published.len() >= 4 {
        paper.published[..4].to_string()
    } else {
        String::new()
    };
    let authors_str = paper.authors.join(", ");
    let meta = if year.is_empty() {
        authors_str.clone()
    } else if authors_str.is_empty() {
        year.clone()
    } else {
        format!("{authors_str} · {year}")
    };

    let methods: Vec<String> = annotation
        .as_ref()
        .map(|a| a.methods.iter().map(|m| m.name.clone()).collect())
        .unwrap_or_default();

    let findings: Vec<String> = annotation
        .as_ref()
        .map(|a| a.findings.iter().map(|f| f.text.clone()).collect())
        .unwrap_or_default();

    let open_problems: Vec<String> = annotation
        .as_ref()
        .map(|a| a.open_problems.clone())
        .unwrap_or_default();

    let arxiv_id = paper.id.clone();

    view! {
        <div class="drawer-header">
            <h2 class="drawer-title">{paper.title.clone()}</h2>
            <button
                class="drawer-close"
                on:click=move |e| on_close.run(e)
                aria-label="Close"
            >
                "✕"
            </button>
        </div>

        <div class="drawer-body">
            // Authors + year
            <p class="drawer-meta">{meta}</p>

            // Abstract
            <section>
                <h3 class="drawer-section-title">"Abstract"</h3>
                <p class="drawer-abstract">{paper.summary.clone()}</p>
            </section>

            // Methods
            <section>
                <h3 class="drawer-section-title">"Methods"</h3>
                {if methods.is_empty() {
                    view! {
                        <p class="text-body text-muted">"No methods annotated."</p>
                    }.into_any()
                } else {
                    view! {
                        <div class="tags-list">
                            {methods.into_iter().map(|m| view! {
                                <span class="tag">{m}</span>
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </section>

            // Findings
            <section>
                <h3 class="drawer-section-title">"Findings"</h3>
                {if findings.is_empty() {
                    view! {
                        <p class="text-body text-muted">"No findings annotated."</p>
                    }.into_any()
                } else {
                    view! {
                        <ul style="list-style: none; display: flex; flex-direction: column; gap: var(--space-xs);">
                            {findings.into_iter().map(|f| view! {
                                <li class="text-body">{f}</li>
                            }).collect_view()}
                        </ul>
                    }.into_any()
                }}
            </section>

            // Open problems
            <section>
                <h3 class="drawer-section-title">"Open Problems"</h3>
                {if open_problems.is_empty() {
                    view! {
                        <p class="text-body text-muted">"No open problems annotated."</p>
                    }.into_any()
                } else {
                    view! {
                        <ul style="list-style: none; display: flex; flex-direction: column; gap: var(--space-xs);">
                            {open_problems.into_iter().map(|p| view! {
                                <li class="text-body">{p}</li>
                            }).collect_view()}
                        </ul>
                    }.into_any()
                }}
            </section>

            // arXiv ID reference
            <p class="text-label text-muted">"arXiv: " {arxiv_id}</p>
        </div>
    }
}

/// Skeleton loading state for the drawer.
#[component]
fn DrawerSkeleton(
    paper_id: String,
    #[prop(into)] on_close: Callback<web_sys::MouseEvent>,
) -> impl IntoView {
    view! {
        <div class="drawer-header">
            <h2 class="drawer-title skeleton skeleton-heading" style="width: 80%; min-height: 24px;">
            </h2>
            <button
                class="drawer-close"
                on:click=move |e| on_close.run(e)
                aria-label="Close"
            >
                "✕"
            </button>
        </div>

        <div class="drawer-body">
            // Authors + year skeleton
            <div class="drawer-meta skeleton skeleton-text" style="width: 60%;"></div>

            // Abstract skeleton
            <section>
                <h3 class="drawer-section-title">"Abstract"</h3>
                <div class="skeleton skeleton-text" style="width: 100%; margin-bottom: 6px;"></div>
                <div class="skeleton skeleton-text" style="width: 95%; margin-bottom: 6px;"></div>
                <div class="skeleton skeleton-text" style="width: 88%;"></div>
            </section>

            // Methods skeleton
            <section>
                <h3 class="drawer-section-title">"Methods"</h3>
                <div class="tags-list">
                    <span class="skeleton tag" style="width: 64px; height: 24px;"></span>
                    <span class="skeleton tag" style="width: 80px; height: 24px;"></span>
                    <span class="skeleton tag" style="width: 56px; height: 24px;"></span>
                </div>
            </section>

            // Findings skeleton
            <section>
                <h3 class="drawer-section-title">"Findings"</h3>
                <div class="skeleton skeleton-text" style="width: 100%; margin-bottom: 6px;"></div>
                <div class="skeleton skeleton-text" style="width: 90%;"></div>
            </section>

            // Open problems skeleton
            <section>
                <h3 class="drawer-section-title">"Open Problems"</h3>
                <div class="skeleton skeleton-text" style="width: 85%; margin-bottom: 6px;"></div>
                <div class="skeleton skeleton-text" style="width: 70%;"></div>
            </section>

            // Paper ID reference (not a skeleton — always available)
            <p class="text-label text-muted">"arXiv: " {paper_id}</p>
        </div>
    }
}
