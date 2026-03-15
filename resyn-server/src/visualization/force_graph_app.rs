use std::collections::HashMap;
use std::time::Instant;

use crossbeam::channel::{Receiver, Sender, unbounded};
use drawers::ValuesSectionDebug;
use eframe::{App, CreationContext};
use egui::{CollapsingHeader, Color32, Context, LayerId, Pos2, ScrollArea, Ui, Vec2};
use egui_graphs::events::Event;
use egui_graphs::{
    DefaultNodeShape, Graph, GraphView, LayoutRandom, LayoutStateRandom, Node,
    SettingsInteraction as GraphSettingsInteraction, SettingsNavigation as GraphSettingsNavigation,
    SettingsStyle as GraphSettingsStyle,
};
use fdg::fruchterman_reingold::{FruchtermanReingold, FruchtermanReingoldConfiguration};
use fdg::nalgebra::{Const, OPoint};
use fdg::{Force, ForceGraph};
use resyn_core::petgraph::Directed;
use resyn_core::petgraph::stable_graph::{DefaultIx, EdgeIndex, NodeIndex};

use resyn_core::datamodels::analysis::PaperAnalysis;
use resyn_core::datamodels::llm_annotation::LlmAnnotation;
use resyn_core::datamodels::paper::Paper;
use resyn_core::utils::strip_version_suffix;
use crate::visualization::drawers;
use crate::visualization::enrichment::{self, TintedEdgeShape};
use crate::visualization::settings;

const EVENTS_LIMIT: usize = 100;

/// Reduced-alpha tint constant for edges (0–255).
const EDGE_TINT_ALPHA: u8 = 120;

type EnrichedGraph = Graph<(), (), Directed, DefaultIx, DefaultNodeShape, TintedEdgeShape>;

type EnrichedEdge =
    egui_graphs::Edge<(), (), Directed, DefaultIx, DefaultNodeShape, TintedEdgeShape>;
type EnrichedNode = Node<(), (), Directed, DefaultIx, DefaultNodeShape>;

pub struct DemoApp {
    g: EnrichedGraph,
    sim: ForceGraph<f32, 2, EnrichedNode, EnrichedEdge>,
    force: FruchtermanReingold<f32, 2>,

    settings_simulation: settings::SettingsSimulation,
    #[allow(dead_code)]
    settings_graph: settings::SettingsGraph,
    settings_interaction: settings::SettingsInteraction,
    settings_navigation: settings::SettingsNavigation,
    settings_style: settings::SettingsStyle,
    settings_analysis: settings::SettingsAnalysis,

    last_events: Vec<String>,

    simulation_stopped: bool,

    fps: f32,
    last_update_time: Instant,
    frames_last_time_span: usize,

    event_publisher: Sender<Event>,
    event_consumer: Receiver<Event>,

    pan: [f32; 2],
    zoom: f32,

    /// Maps NodeIndex to arxiv_id for enrichment lookups.
    node_id_map: HashMap<NodeIndex<u32>, String>,
    /// Maps arxiv_id to paper title for tooltip display.
    node_title_map: HashMap<String, String>,
    /// LLM annotations keyed by arxiv_id.
    annotations: HashMap<String, LlmAnnotation>,
    /// TF-IDF analyses keyed by arxiv_id.
    analyses: HashMap<String, PaperAnalysis>,
}

impl DemoApp {
    pub fn new(
        _: &CreationContext<'_>,
        petgraph_graph: resyn_core::petgraph::stable_graph::StableGraph<Paper, f32>,
        annotations: HashMap<String, LlmAnnotation>,
        analyses: HashMap<String, PaperAnalysis>,
    ) -> Self {
        // Build lookup maps from the weighted graph (while Paper data is still available).
        let mut node_id_map: HashMap<NodeIndex<u32>, String> = HashMap::new();
        let mut node_title_map: HashMap<String, String> = HashMap::new();
        for idx in petgraph_graph.node_indices() {
            let arxiv_id = strip_version_suffix(&petgraph_graph[idx].id);
            node_id_map.insert(idx, arxiv_id.clone());
            node_title_map.insert(arxiv_id, petgraph_graph[idx].title.clone());
        }

        // Strip weights to produce the lightweight graph expected by egui_graphs.
        let stripped = petgraph_graph.map(|_, _| (), |_, _| ());
        let mut g: EnrichedGraph = Graph::from(&stripped);

        let settings_graph = settings::SettingsGraph::default();
        let settings_simulation = settings::SettingsSimulation::default();
        let mut force = init_force(&settings_simulation);
        let mut sim = fdg::init_force_graph_uniform(g.g.clone(), 1.0);
        force.apply(&mut sim);
        g.g.node_weights_mut().for_each(|node| {
            let point: fdg::nalgebra::OPoint<f32, fdg::nalgebra::Const<2>> =
                sim.node_weight(node.id()).unwrap().1;
            node.set_location(Pos2::new(point.coords.x, point.coords.y));
        });

        let (event_publisher, event_consumer) = unbounded();
        let graph = g.clone();
        Self {
            g: graph,
            sim,
            force,

            event_consumer,
            event_publisher,

            settings_graph,
            settings_simulation,

            settings_interaction: settings::SettingsInteraction::default(),
            settings_navigation: settings::SettingsNavigation::default(),
            settings_style: settings::SettingsStyle::default(),
            settings_analysis: settings::SettingsAnalysis::default(),

            last_events: Vec::default(),

            simulation_stopped: false,

            fps: 0.,
            last_update_time: Instant::now(),
            frames_last_time_span: 0,

            pan: [0., 0.],
            zoom: 0.,

            node_id_map,
            node_title_map,
            annotations,
            analyses,
        }
    }

    /// Applies node colors/sizes and edge tints based on enriched view state.
    /// Called after `sync()` to apply per-frame visual enrichment.
    fn apply_enrichment(&mut self) {
        let enriched = self.settings_analysis.enriched_view;

        // --- Node enrichment ---
        self.g.g.node_weights_mut().for_each(|node| {
            if enriched {
                let arxiv_id = self
                    .node_id_map
                    .get(&node.id())
                    .cloned()
                    .unwrap_or_default();
                if let Some(ann) = self.annotations.get(&arxiv_id) {
                    node.set_color(enrichment::paper_type_to_color(&ann.paper_type));
                    node.display_mut().radius =
                        enrichment::finding_strength_radius(&ann.findings, enrichment::BASE_RADIUS);
                } else {
                    node.set_color(enrichment::GRAY_UNANALYZED);
                    node.display_mut().radius = enrichment::BASE_RADIUS;
                }
            } else {
                node.set_color(enrichment::DEFAULT_NODE_COLOR);
                node.display_mut().radius = enrichment::BASE_RADIUS;
            }
        });

        // --- Edge tinting ---
        // Collect edge indices first to avoid borrowing `self.g` twice.
        let edge_indices: Vec<EdgeIndex<DefaultIx>> = self.g.g.edge_indices().collect();

        for edge_idx in edge_indices {
            let tint = if enriched {
                // Determine source node's paper type color.
                let src_color = self
                    .g
                    .g
                    .edge_endpoints(edge_idx)
                    .and_then(|(src_node_idx, _)| {
                        let arxiv_id = self.node_id_map.get(&src_node_idx)?;
                        let ann = self.annotations.get(arxiv_id)?;
                        Some(enrichment::paper_type_to_color(&ann.paper_type))
                    });
                src_color
                    .map(|c| Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), EDGE_TINT_ALPHA))
            } else {
                None
            };

            if let Some(edge) = self.g.g.edge_weight_mut(edge_idx) {
                edge.display_mut().color_override = tint;
            }
        }
    }

    /// Returns the arxiv_id of the node currently hovered by the pointer, or `None`.
    ///
    /// Uses pointer-to-graph-space conversion with pan/zoom tracked from events.
    fn find_hovered_node(&self, ctx: &Context) -> Option<String> {
        let screen_pos = ctx.input(|i| i.pointer.hover_pos())?;

        // Convert from screen space to graph (canvas) space.
        // Pan is in screen pixels; zoom is the scale factor.
        let zoom = if self.zoom == 0. { 1.0 } else { self.zoom };
        let graph_pos = Pos2::new(
            (screen_pos.x - self.pan[0]) / zoom,
            (screen_pos.y - self.pan[1]) / zoom,
        );

        let mut closest: Option<(f32, String)> = None;
        for node in self.g.g.node_weights() {
            let loc = node.location();
            let radius = node.display().radius;
            let dist = (graph_pos - loc).length();
            if dist <= radius * 1.5 {
                // 1.5x hit tolerance for usability
                let arxiv_id = self
                    .node_id_map
                    .get(&node.id())
                    .cloned()
                    .unwrap_or_default();
                match &closest {
                    None => closest = Some((dist, arxiv_id)),
                    Some((prev_dist, _)) if dist < *prev_dist => {
                        closest = Some((dist, arxiv_id));
                    }
                    _ => {}
                }
            }
        }

        closest.map(|(_, id)| id)
    }

    /// applies forces if simulation is running
    fn update_simulation(&mut self) {
        if self.simulation_stopped {
            return;
        }

        self.force.apply(&mut self.sim);
    }

    /// sync locations computed by the simulation with egui_graphs::Graph nodes.
    fn sync(&mut self) {
        self.g.g.node_weights_mut().for_each(|node| {
            let sim_computed_point: OPoint<f32, Const<2>> =
                self.sim.node_weight(node.id()).unwrap().1;
            node.set_location(Pos2::new(
                sim_computed_point.coords.x,
                sim_computed_point.coords.y,
            ));
        });
    }

    fn update_fps(&mut self) {
        self.frames_last_time_span += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update_time);
        if elapsed.as_secs() >= 1 {
            self.last_update_time = now;
            self.fps = self.frames_last_time_span as f32 / elapsed.as_secs_f32();
            self.frames_last_time_span = 0;
        }
    }

    fn handle_events(&mut self) {
        self.event_consumer.try_iter().for_each(|e| {
            if self.last_events.len() > EVENTS_LIMIT {
                self.last_events.remove(0);
            }
            self.last_events.push(serde_json::to_string(&e).unwrap());

            match e {
                Event::Pan(payload) => self.pan = payload.new_pan,
                Event::Zoom(payload) => self.zoom = payload.new_zoom,
                Event::NodeMove(payload) => {
                    let node_id = NodeIndex::new(payload.id);

                    self.sim.node_weight_mut(node_id).unwrap().1.coords.x = payload.new_pos[0];
                    self.sim.node_weight_mut(node_id).unwrap().1.coords.y = payload.new_pos[1];
                }
                _ => {}
            }
        });
    }

    fn draw_section_simulation(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.style_mut().spacing.item_spacing = Vec2::new(0., 0.);
            ui.label("Force-Directed Simulation is done with ");
            ui.hyperlink_to("fdg project", "https://github.com/grantshandy/fdg");
        });

        ui.separator();

        ui.add_space(10.);

        drawers::draw_simulation_config_sliders(
            ui,
            drawers::ValuesConfigSlidersSimulation {
                dt: self.settings_simulation.dt,
                cooloff_factor: self.settings_simulation.cooloff_factor,
                scale: self.settings_simulation.scale,
            },
            |delta_dt: f32, delta_cooloff_factor: f32, delta_scale: f32| {
                self.settings_simulation.dt += delta_dt;
                self.settings_simulation.cooloff_factor += delta_cooloff_factor;
                self.settings_simulation.scale += delta_scale;

                self.force = init_force(&self.settings_simulation);
            },
        );

        ui.add_space(10.);
    }

    fn draw_section_analysis(&mut self, ui: &mut Ui) {
        // Toggle checkbox
        ui.checkbox(&mut self.settings_analysis.enriched_view, "Enriched view");
        ui.label("Color nodes by paper type and size by finding strength.");

        if self.settings_analysis.enriched_view {
            ui.add_space(5.);
            ui.label("Paper Types:");

            // Color legend
            for (label, color) in [
                (
                    "Theoretical",
                    enrichment::paper_type_to_color("theoretical"),
                ),
                (
                    "Experimental",
                    enrichment::paper_type_to_color("experimental"),
                ),
                ("Review", enrichment::paper_type_to_color("review")),
                (
                    "Computational",
                    enrichment::paper_type_to_color("computational"),
                ),
                ("Not analyzed", enrichment::GRAY_UNANALYZED),
            ] {
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, color);
                    ui.label(label);
                });
            }

            ui.add_space(5.);

            // Stats
            let total = self.node_id_map.len();
            let analyzed = self
                .node_id_map
                .values()
                .filter(|id| self.annotations.contains_key(*id))
                .count();
            ui.label(format!("{analyzed}/{total} papers analyzed"));
        }
    }

    fn draw_section_widget(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Navigation")
            .default_open(true)
            .show(ui, |ui| {
                if ui
                    .checkbox(
                        &mut self.settings_navigation.fit_to_screen_enabled,
                        "fit_to_screen",
                    )
                    .changed()
                    && self.settings_navigation.fit_to_screen_enabled
                {
                    self.settings_navigation.zoom_and_pan_enabled = false
                };
                ui.label("Enable fit to screen to fit the graph to the screen on every frame.");

                ui.add_space(5.);

                ui.add_enabled_ui(!self.settings_navigation.fit_to_screen_enabled, |ui| {
                    ui.vertical(|ui| {
                        ui.checkbox(
                            &mut self.settings_navigation.zoom_and_pan_enabled,
                            "zoom_and_pan",
                        );
                        ui.label("Zoom with ctrl + mouse wheel, pan with middle mouse drag.");
                    })
                    .response
                    .on_disabled_hover_text("disable fit_to_screen to enable zoom_and_pan");
                });
            });

        CollapsingHeader::new("Style").show(ui, |ui| {
            ui.checkbox(&mut self.settings_style.labels_always, "labels_always");
            ui.label("Wheter to show labels always or when interacted only.");
        });

        CollapsingHeader::new("Interaction").show(ui, |ui| {
                if ui.checkbox(&mut self.settings_interaction.dragging_enabled, "dragging_enabled").clicked() && self.settings_interaction.dragging_enabled {
                    self.settings_interaction.node_clicking_enabled = true;
                };
                ui.label("To drag use LMB click + drag on a node.");

                ui.add_space(5.);

                ui.add_enabled_ui(!(self.settings_interaction.dragging_enabled || self.settings_interaction.node_selection_enabled || self.settings_interaction.node_selection_multi_enabled), |ui| {
                    ui.vertical(|ui| {
                        ui.checkbox(&mut self.settings_interaction.node_clicking_enabled, "node_clicking_enabled");
                        ui.label("Check click events in last events");
                    }).response.on_disabled_hover_text("node click is enabled when any of the interaction is also enabled");
                });

                ui.add_space(5.);

                ui.add_enabled_ui(!self.settings_interaction.node_selection_multi_enabled, |ui| {
                    ui.vertical(|ui| {
                        if ui.checkbox(&mut self.settings_interaction.node_selection_enabled, "node_selection_enabled").clicked() && self.settings_interaction.node_selection_enabled {
                            self.settings_interaction.node_clicking_enabled = true;
                        };
                        ui.label("Enable select to select nodes with LMB click. If node is selected clicking on it again will deselect it.");
                    }).response.on_disabled_hover_text("node_selection_multi_enabled enables select");
                });

                if ui.checkbox(&mut self.settings_interaction.node_selection_multi_enabled, "node_selection_multi_enabled").changed() && self.settings_interaction.node_selection_multi_enabled {
                    self.settings_interaction.node_clicking_enabled = true;
                    self.settings_interaction.node_selection_enabled = true;
                }
                ui.label("Enable multiselect to select multiple nodes.");

                ui.add_space(5.);

                ui.add_enabled_ui(!(self.settings_interaction.edge_selection_enabled || self.settings_interaction.edge_selection_multi_enabled), |ui| {
                    ui.vertical(|ui| {
                        ui.checkbox(&mut self.settings_interaction.edge_clicking_enabled, "edge_clicking_enabled");
                        ui.label("Check click events in last events");
                    }).response.on_disabled_hover_text("edge click is enabled when any of the interaction is also enabled");
                });

                ui.add_space(5.);

                ui.add_enabled_ui(!self.settings_interaction.edge_selection_multi_enabled, |ui| {
                    ui.vertical(|ui| {
                        if ui.checkbox(&mut self.settings_interaction.edge_selection_enabled, "edge_selection_enabled").clicked() && self.settings_interaction.edge_selection_enabled {
                            self.settings_interaction.edge_clicking_enabled = true;
                        };
                        ui.label("Enable select to select edges with LMB click. If edge is selected clicking on it again will deselect it.");
                    }).response.on_disabled_hover_text("edge_selection_multi_enabled enables select");
                });

                if ui.checkbox(&mut self.settings_interaction.edge_selection_multi_enabled, "edge_selection_multi_enabled").changed() && self.settings_interaction.edge_selection_multi_enabled {
                    self.settings_interaction.edge_clicking_enabled = true;
                    self.settings_interaction.edge_selection_enabled = true;
                }
                ui.label("Enable multiselect to select multiple edges.");
            });

        CollapsingHeader::new("Selected")
            .default_open(true)
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .max_height(200.)
                    .show(ui, |ui| {
                        self.g.selected_nodes().iter().for_each(|node| {
                            ui.label(format!("{node:?}"));
                        });
                        self.g.selected_edges().iter().for_each(|edge| {
                            ui.label(format!("{edge:?}"));
                        });
                    });
            });

        CollapsingHeader::new("Last Events")
            .default_open(true)
            .show(ui, |ui| {
                if ui.button("clear").clicked() {
                    self.last_events.clear();
                }
                ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        self.last_events.iter().rev().for_each(|event| {
                            ui.label(event);
                        });
                    });
            });
    }

    fn draw_section_debug(&self, ui: &mut Ui) {
        drawers::draw_section_debug(
            ui,
            ValuesSectionDebug {
                zoom: self.zoom,
                pan: self.pan,
                fps: self.fps,
            },
        );
    }
}

impl App for DemoApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::SidePanel::right("right_panel")
            .min_width(250.)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    CollapsingHeader::new("Simulation")
                        .default_open(true)
                        .show(ui, |ui| self.draw_section_simulation(ui));

                    ui.add_space(10.);

                    CollapsingHeader::new("Analysis")
                        .default_open(true)
                        .show(ui, |ui| self.draw_section_analysis(ui));

                    ui.add_space(10.);

                    egui::CollapsingHeader::new("Debug")
                        .default_open(true)
                        .show(ui, |ui| self.draw_section_debug(ui));

                    ui.add_space(10.);

                    CollapsingHeader::new("Widget")
                        .default_open(true)
                        .show(ui, |ui| self.draw_section_widget(ui));
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let settings_interaction = &GraphSettingsInteraction::new()
                .with_node_selection_enabled(self.settings_interaction.node_selection_enabled)
                .with_node_selection_multi_enabled(
                    self.settings_interaction.node_selection_multi_enabled,
                )
                .with_dragging_enabled(self.settings_interaction.dragging_enabled)
                .with_node_clicking_enabled(self.settings_interaction.node_clicking_enabled)
                .with_edge_clicking_enabled(self.settings_interaction.edge_clicking_enabled)
                .with_edge_selection_enabled(self.settings_interaction.edge_selection_enabled)
                .with_edge_selection_multi_enabled(
                    self.settings_interaction.edge_selection_multi_enabled,
                );
            let settings_navigation = &GraphSettingsNavigation::new()
                .with_zoom_and_pan_enabled(self.settings_navigation.zoom_and_pan_enabled)
                .with_fit_to_screen_enabled(self.settings_navigation.fit_to_screen_enabled)
                .with_zoom_speed(self.settings_navigation.zoom_speed);
            let settings_style =
                &GraphSettingsStyle::new().with_labels_always(self.settings_style.labels_always);
            ui.add(
                &mut GraphView::<
                    _,
                    _,
                    _,
                    _,
                    DefaultNodeShape,
                    TintedEdgeShape,
                    LayoutStateRandom,
                    LayoutRandom,
                >::new(&mut self.g)
                .with_interactions(settings_interaction)
                .with_navigations(settings_navigation)
                .with_styles(settings_style)
                .with_events(&self.event_publisher),
            );
        });

        // Tooltip: rendered after CentralPanel so it overlaps it correctly.
        if self.settings_analysis.enriched_view
            && let Some(hovered_id) = self.find_hovered_node(ctx)
        {
            let title = self
                .node_title_map
                .get(&hovered_id)
                .map(|t| t.as_str())
                .unwrap_or(hovered_id.as_str());

            let annotation = self.annotations.get(&hovered_id).cloned();
            let analysis = self.analyses.get(&hovered_id).cloned();
            let title_owned = title.to_owned();
            let hovered_id_clone = hovered_id.clone();

            egui::show_tooltip_at_pointer(
                ctx,
                LayerId::background(),
                egui::Id::new("node_tooltip"),
                |ui| {
                    if let Some(ann) = annotation {
                        ui.strong(format!("{} [{}]", title_owned, &ann.paper_type));
                        if let Some(analysis) = analysis {
                            let kw_count = analysis.top_terms.len().min(5);
                            if kw_count > 0 {
                                ui.label(format!(
                                    "Keywords: {}",
                                    analysis.top_terms[..kw_count].join(", ")
                                ));
                            }
                        }
                        if let Some(method) = ann.methods.first() {
                            ui.label(format!("Method: {} ({})", method.name, method.category));
                        }
                    } else {
                        ui.strong(title_owned);
                        ui.label(format!("ID: {hovered_id_clone}"));
                        ui.label("Not analyzed");
                    }
                },
            );
        }

        self.handle_events();
        self.sync();
        self.apply_enrichment();
        self.update_simulation();
        self.update_fps();
    }
}

fn init_force(settings: &settings::SettingsSimulation) -> FruchtermanReingold<f32, 2> {
    FruchtermanReingold {
        conf: FruchtermanReingoldConfiguration {
            dt: settings.dt,
            cooloff_factor: settings.cooloff_factor,
            scale: settings.scale,
        },
        ..Default::default()
    }
}
