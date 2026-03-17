use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::layout::{drawer::Drawer, sidebar::Sidebar};
use crate::pages::{
    dashboard::Dashboard, gaps::GapsPanel, methods::MethodsPanel, open_problems::OpenProblemsPanel,
    papers::PapersPanel,
};

/// App-level context: selected paper ID drives the detail drawer.
/// None = drawer closed.
#[derive(Clone, Copy)]
pub struct SelectedPaper(pub RwSignal<Option<String>>);

/// App-level context: sidebar collapsed state.
#[derive(Clone, Copy)]
pub struct SidebarCollapsed(pub RwSignal<bool>);

#[component]
pub fn App() -> impl IntoView {
    let selected_paper: RwSignal<Option<String>> = RwSignal::new(None);
    let sidebar_collapsed: RwSignal<bool> = RwSignal::new(false);

    provide_context(SelectedPaper(selected_paper));
    provide_context(SidebarCollapsed(sidebar_collapsed));

    view! {
        <Router>
            <div class="app-shell">
                <Sidebar/>
                <main class="content-area">
                    <Routes fallback=|| view! { <p class="text-muted">"Page not found."</p> }>
                        <Route path=path!("/") view=Dashboard/>
                        <Route path=path!("/papers") view=PapersPanel/>
                        <Route path=path!("/gaps") view=GapsPanel/>
                        <Route path=path!("/problems") view=OpenProblemsPanel/>
                        <Route path=path!("/methods") view=MethodsPanel/>
                    </Routes>
                </main>
                <Drawer/>
            </div>
        </Router>
    }
}
