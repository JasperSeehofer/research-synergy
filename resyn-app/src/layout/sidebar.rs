use leptos::prelude::*;
use leptos_router::components::A;

use crate::app::SidebarCollapsed;
use crate::components::crawl_progress::CrawlProgress;

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
                <CrawlProgress collapsed=collapsed/>
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
