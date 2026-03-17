use leptos::prelude::*;
use leptos::web_sys;

use crate::app::SelectedPaper;

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

    let close = move |_| selected.set(None);

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
