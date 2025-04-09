use eframe::{App, CreationContext};
use egui::Context;
use egui_graphs::{
    DefaultGraphView, Graph, SettingsInteraction, SettingsNavigation, SettingsStyle,
};
use petgraph::stable_graph::StableGraph;

pub struct InteractiveApp {
    g: Graph,
}

impl InteractiveApp {
    pub fn new(_: &CreationContext<'_>, graph: StableGraph<(), ()>) -> Self {
        let g: Graph = Graph::from(&graph);
        Self { g }
    }
}

impl App for InteractiveApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let interaction_settings = &SettingsInteraction::new()
                .with_dragging_enabled(true)
                .with_node_clicking_enabled(true)
                .with_node_selection_enabled(true)
                .with_node_selection_multi_enabled(true)
                .with_edge_clicking_enabled(true)
                .with_edge_selection_enabled(true)
                .with_edge_selection_multi_enabled(true);
            let style_settings = &SettingsStyle::new().with_labels_always(true);
            let navigation_settings = &SettingsNavigation::new()
                .with_fit_to_screen_enabled(false)
                .with_zoom_and_pan_enabled(true);

            ui.add(
                &mut DefaultGraphView::new(&mut self.g)
                    .with_styles(style_settings)
                    .with_interactions(interaction_settings)
                    .with_navigations(navigation_settings),
            );
        });
    }
}
