use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::search_bar::GlobalSearchBar;
use crate::layout::{drawer::Drawer, sidebar::Sidebar};
use crate::pages::{
    dashboard::Dashboard, gaps::GapsPanel, graph::GraphPage, methods::MethodsPanel,
    open_problems::OpenProblemsPanel, papers::PapersPanel,
};

/// Which tab is active in the paper detail drawer.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum DrawerTab {
    #[default]
    Overview,
    Source,
}

/// Request to open the drawer for a specific paper, optionally on a specific tab
/// with provenance highlight context.
#[derive(Clone, Debug, Default)]
pub struct DrawerOpenRequest {
    pub paper_id: String,
    pub initial_tab: DrawerTab,
    pub highlight_snippet: Option<String>,
    pub highlight_section: Option<String>,
}

/// App-level context: selected paper drives the detail drawer.
/// None = drawer closed.
#[derive(Clone, Copy)]
pub struct SelectedPaper(pub RwSignal<Option<DrawerOpenRequest>>);

/// App-level context: sidebar collapsed state.
#[derive(Clone, Copy)]
pub struct SidebarCollapsed(pub RwSignal<bool>);

/// Request to pan the graph viewport to a specific paper node and highlight it.
/// Set by GlobalSearchBar when user selects a result on the graph page.
/// Consumed by GraphPage to trigger lerp animation + pulse glow.
#[derive(Clone, Debug, Default)]
pub struct SearchPanRequest {
    pub paper_id: String,
}

/// App-level context: search-triggered pan request for the graph page.
/// None = no pending pan. Cleared by GraphPage after animation completes.
#[derive(Clone, Copy)]
pub struct SearchPanTrigger(pub RwSignal<Option<SearchPanRequest>>);

#[component]
pub fn App() -> impl IntoView {
    let selected_paper: RwSignal<Option<DrawerOpenRequest>> = RwSignal::new(None);
    let sidebar_collapsed: RwSignal<bool> = RwSignal::new(false);
    let search_pan: RwSignal<Option<SearchPanRequest>> = RwSignal::new(None);

    provide_context(SelectedPaper(selected_paper));
    provide_context(SidebarCollapsed(sidebar_collapsed));
    provide_context(SearchPanTrigger(search_pan));

    view! {
        <Router>
            <div class="app-shell">
                <Sidebar/>
                <main class="content-area">
                    <div class="top-bar">
                        <GlobalSearchBar/>
                    </div>
                    <div class="content-scroll">
                        <Routes fallback=|| view! { <p class="text-muted">"Page not found."</p> }>
                            <Route path=path!("/") view=Dashboard/>
                            <Route path=path!("/papers") view=PapersPanel/>
                            <Route path=path!("/gaps") view=GapsPanel/>
                            <Route path=path!("/problems") view=OpenProblemsPanel/>
                            <Route path=path!("/methods") view=MethodsPanel/>
                            <Route path=path!("/graph") view=GraphPage/>
                        </Routes>
                    </div>
                </main>
                <Drawer/>
            </div>
        </Router>
    }
}
