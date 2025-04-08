use eframe;
use egui;
use egui_graphs;
pub struct BasicApp {
    g: egui_graphs::Graph,
}

impl BasicApp {
    fn new(_: &eframe::CreationContext<'_>, graph: egui_graphs::Graph) -> Self {
        Self { g: graph }
    }
}

impl eframe::App for BasicApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut egui_graphs::GraphView::<
                (),
                (),
                petgraph::Directed,
                u32,
                egui_graphs::DefaultNodeShape,
                egui_graphs::DefaultEdgeShape,
                egui_graphs::LayoutStateHierarchical,
                egui_graphs::LayoutHierarchical,
            >::new(&mut self.g));
        });
    }
}
