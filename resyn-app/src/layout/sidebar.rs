use leptos::prelude::*;
use leptos_router::components::A;

use crate::app::SidebarCollapsed;

/// Collapsible sidebar with navigation links and a crawl progress footer.
#[component]
pub fn Sidebar() -> impl IntoView {
    let SidebarCollapsed(collapsed) = expect_context::<SidebarCollapsed>();

    let sidebar_class = move || {
        if collapsed.get() {
            "sidebar rail"
        } else {
            "sidebar expanded"
        }
    };

    let toggle_label = move || {
        if collapsed.get() {
            "Expand sidebar"
        } else {
            "Collapse sidebar"
        }
    };

    view! {
        <nav class=sidebar_class role="navigation" aria-label="Main navigation">
            <div class="sidebar-header">
                <span class="sidebar-title">"ReSyn"</span>
                <button
                    class="sidebar-toggle"
                    on:click=move |_| collapsed.update(|v| *v = !*v)
                    aria-expanded=move || (!collapsed.get()).to_string()
                    aria-label=toggle_label
                >
                    {move || if collapsed.get() { "›" } else { "‹" }}
                </button>
            </div>

            <div class="sidebar-nav">
                <NavItem href="/" icon="◈" label="Dashboard"/>
                <NavItem href="/papers" icon="⊞" label="Papers"/>
                <NavItem href="/gaps" icon="⊘" label="Gaps"/>
                <NavItem href="/problems" icon="?" label="Open Problems"/>
                <NavItem href="/methods" icon="⊙" label="Methods"/>
            </div>

            <div class="sidebar-footer">
                <CrawlProgressFooter collapsed=collapsed/>
            </div>
        </nav>
    }
}

/// A single navigation item. Renders icon + optional label (hidden in rail mode via CSS).
#[component]
fn NavItem(href: &'static str, icon: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <A href=href>
            <span class="nav-icon">{icon}</span>
            <span class="nav-label">{label}</span>
            // Tooltip shown in rail mode on hover
            <span class="nav-tooltip">{label}</span>
        </A>
    }
}

/// Crawl progress section pinned to the sidebar footer.
/// Shows idle summary, running stats, or spinning indicator in rail mode.
#[component]
fn CrawlProgressFooter(collapsed: RwSignal<bool>) -> impl IntoView {
    view! {
        <div class="crawl-progress-idle text-label text-muted">
            {move || {
                if collapsed.get() {
                    view! { <div class="crawl-spinner"></div> }.into_any()
                } else {
                    view! {
                        <div>
                            <p class="crawl-progress-idle">"No active crawl"</p>
                            <CrawlForm/>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// Crawl start form embedded in the sidebar footer (expanded state only).
#[component]
fn CrawlForm() -> impl IntoView {
    let paper_id = RwSignal::new(String::new());
    let depth = RwSignal::new(3u8);
    let source = RwSignal::new("arxiv".to_string());

    view! {
        <form class="crawl-form" on:submit=|e| e.prevent_default()>
            <div>
                <label class="form-label" for="paper-id-input">"Paper ID"</label>
                <input
                    id="paper-id-input"
                    class="form-input"
                    type="text"
                    placeholder="e.g. 2503.18887"
                    prop:value=paper_id
                    on:input=move |e| paper_id.set(event_target_value(&e))
                />
            </div>
            <div>
                <label class="form-label" for="depth-select">"Depth"</label>
                <select
                    id="depth-select"
                    class="form-select"
                    on:change=move |e| {
                        if let Ok(v) = event_target_value(&e).parse::<u8>() {
                            depth.set(v);
                        }
                    }
                >
                    {(1u8..=10).map(|d| view! {
                        <option value=d.to_string() selected=move || depth.get() == d>
                            {d.to_string()}
                        </option>
                    }).collect_view()}
                </select>
            </div>
            <div>
                <label class="form-label" for="source-select">"Source"</label>
                <select
                    id="source-select"
                    class="form-select"
                    on:change=move |e| source.set(event_target_value(&e))
                >
                    <option value="arxiv" selected=move || source.get() == "arxiv">"arXiv"</option>
                    <option value="inspirehep" selected=move || source.get() == "inspirehep">"InspireHEP"</option>
                </select>
            </div>
            <button
                type="submit"
                class="btn-primary"
                disabled=move || paper_id.get().is_empty()
            >
                "Start Crawl"
            </button>
        </form>
    }
}
