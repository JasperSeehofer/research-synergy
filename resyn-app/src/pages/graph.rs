use leptos::html::Canvas;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::app::{DrawerOpenRequest, SelectedPaper};
use crate::components::graph_controls::{GraphControls, TemporalSlider};
use crate::graph::interaction::{self, InteractionState};
use crate::graph::layout_state::GraphState;
use crate::graph::make_renderer;
use crate::graph::renderer::{Renderer, Viewport};
use crate::graph::worker_bridge::WorkerBridge;
use crate::server_fns::graph::get_graph_data;
use resyn_worker::{LayoutInput, LayoutOutput, NodeData};

// ── Tooltip data ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct TooltipData {
    pub text: String,
    pub x: f64,
    pub y: f64,
}

// ── Shared render state (outside Leptos reactive graph) ─────────────────────

struct RenderState {
    graph: GraphState,
    viewport: Viewport,
    interaction: InteractionState,
    /// Whether the node under mousedown was already pinned before this interaction
    was_already_pinned: bool,
    /// Canvas-space position at mousedown — used to distinguish click from drag
    drag_start_x: f64,
    drag_start_y: f64,
}

// ── RAF handle with cancel support ──────────────────────────────────────────

struct RafHandle {
    cancelled: Arc<AtomicBool>,
}

impl RafHandle {
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

// ── GraphPage component ──────────────────────────────────────────────────────

#[component]
pub fn GraphPage() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();

    // Overlay toggle signals — Leptos reactive (drive UI buttons)
    let show_contradictions: RwSignal<bool> = RwSignal::new(true);
    let show_bridges: RwSignal<bool> = RwSignal::new(true);
    let simulation_running: RwSignal<bool> = RwSignal::new(true);

    // Zoom button signals: increment a counter to trigger zoom from RAF loop
    let zoom_in_count: RwSignal<u32> = RwSignal::new(0);
    let zoom_out_count: RwSignal<u32> = RwSignal::new(0);

    // Temporal filter signals
    let temporal_min: RwSignal<u32> = RwSignal::new(2000);
    let temporal_max: RwSignal<u32> = RwSignal::new(2026);
    let year_bounds: RwSignal<(u32, u32)> = RwSignal::new((2000, 2026));
    let visible_count: RwSignal<(usize, usize)> = RwSignal::new((0, 0));

    // Tooltip overlay signal
    let tooltip_signal: RwSignal<Option<TooltipData>> = RwSignal::new(None);

    // Fetch graph data from server
    let graph_resource = Resource::new(|| (), |_| get_graph_data());

    // SelectedPaper context — for drawer integration
    let SelectedPaper(selected_paper) = expect_context::<SelectedPaper>();

    // Effect: fires when canvas is mounted and graph data arrives
    Effect::new(move |_| {
        let Some(canvas_el) = canvas_ref.get() else {
            return;
        };
        let canvas: web_sys::HtmlCanvasElement = canvas_el;
        let Some(Ok(data)) = graph_resource.get() else {
            return;
        };

        // Size canvas to its CSS container (accounting for device pixel ratio)
        let dpr = web_sys::window().unwrap().device_pixel_ratio();
        let css_width = canvas.offset_width() as f64;
        let css_height = canvas.offset_height() as f64;
        canvas.set_width((css_width * dpr) as u32);
        canvas.set_height((css_height * dpr) as u32);

        let mut viewport = Viewport::new(css_width, css_height);
        let mut graph_state = GraphState::from_graph_data(data);
        // Fit initial spread into visible canvas area so nodes are on-screen
        if !graph_state.nodes.is_empty() {
            let spread = (graph_state.nodes.len() as f64).sqrt() * 15.0;
            let fit_scale = (css_width.min(css_height) * 0.4 / spread).min(1.0);
            viewport.scale = fit_scale;
        }
        let renderer = make_renderer(&canvas, graph_state.nodes.len());
        // Sync initial toggle values from Leptos signals
        graph_state.show_contradictions = show_contradictions.get_untracked();
        graph_state.show_bridges = show_bridges.get_untracked();
        graph_state.simulation_running = simulation_running.get_untracked();

        // Initialize temporal signals from graph year bounds
        let year_min = graph_state.temporal_min_year;
        let year_max = graph_state.temporal_max_year;
        temporal_min.set(year_min);
        temporal_max.set(year_max);
        year_bounds.set((year_min, year_max));

        let state = Rc::new(RefCell::new(RenderState {
            graph: graph_state,
            viewport,
            interaction: InteractionState::Idle,
            was_already_pinned: false,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
        }));

        let renderer_rc: Rc<RefCell<Box<dyn Renderer>>> = Rc::new(RefCell::new(renderer));

        // Force layout runs inline on the main thread (1 tick per RAF frame).
        // The Web Worker bridge has waker issues with gloo-worker's
        // ReactorBridge that prevent outputs from being received. For 400+
        // nodes, a single Barnes-Hut tick per frame is fast enough (<2ms).
        // The bridge/worker infrastructure is kept for future off-thread use.
        let pinned_bridge: PinnedBridge = {
            let bridge = WorkerBridge::new();
            Rc::new(RefCell::new(Box::pin(bridge.bridge)))
        };
        let output_buf: Rc<RefCell<Option<LayoutOutput>>> = Rc::new(RefCell::new(None));

        // Set up ResizeObserver to handle canvas resize
        setup_resize_observer(&canvas, state.clone(), renderer_rc.clone());

        // Attach event listeners to canvas
        attach_event_listeners(
            &canvas,
            state.clone(),
            pinned_bridge.clone(),
            tooltip_signal,
            selected_paper,
        );

        // Start RAF render loop
        let handle = start_render_loop(
            state.clone(),
            renderer_rc.clone(),
            pinned_bridge.clone(),
            output_buf.clone(),
            show_contradictions,
            show_bridges,
            simulation_running,
            zoom_in_count,
            zoom_out_count,
            temporal_min,
            temporal_max,
            visible_count,
        );

        on_cleanup(move || handle.cancel());
    });

    view! {
        <div class="graph-page">
            <Suspense fallback=move || view! { <div class="graph-loading">"Loading graph..."</div> }>
                {move || graph_resource.get().map(|result| {
                    match result {
                        Err(_) => view! {
                            <div class="error-banner">
                                "Failed to load graph. Check the server connection and reload the page."
                            </div>
                        }.into_any(),
                        Ok(data) if data.nodes.is_empty() => view! {
                            <div class="empty-state">
                                <h2>"No graph data"</h2>
                                <p>"Run a crawl to build the citation graph. Start a crawl from the Dashboard."</p>
                            </div>
                        }.into_any(),
                        Ok(_) => view! {
                            <canvas
                                node_ref=canvas_ref
                                class="graph-canvas"
                                role="img"
                                aria-label="Citation graph"
                            />
                            <GraphControls
                                show_contradictions=show_contradictions
                                show_bridges=show_bridges
                                simulation_running=simulation_running
                                zoom_in_count=zoom_in_count
                                zoom_out_count=zoom_out_count
                                temporal_min=temporal_min
                                temporal_max=temporal_max
                                year_bounds=year_bounds
                                visible_count=visible_count
                            />
                            <TemporalSlider
                                temporal_min=temporal_min
                                temporal_max=temporal_max
                                year_bounds=year_bounds
                            />
                            {move || tooltip_signal.get().map(|t| view! {
                                <div
                                    class="graph-tooltip"
                                    style=format!("left: {}px; top: {}px;", t.x + 12.0, t.y - 12.0)
                                >
                                    {t.text}
                                </div>
                            })}
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}

// ── Type aliases ─────────────────────────────────────────────────────────────

type ClosureSlot = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

type PinnedBridge = Rc<
    RefCell<
        std::pin::Pin<
            Box<gloo_worker::reactor::ReactorBridge<resyn_worker::ForceLayoutWorker>>,
        >,
    >,
>;

// ── Helper: build LayoutInput from graph state ────────────────────────────────

fn build_layout_input(graph: &GraphState, width: f64, height: f64) -> LayoutInput {
    let nodes: Vec<NodeData> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let (vx, vy) = graph.velocities.get(i).copied().unwrap_or((0.0, 0.0));
            NodeData { x: n.x, y: n.y, vx, vy, mass: 1.0, pinned: n.pinned }
        })
        .collect();
    let edges: Vec<(usize, usize)> =
        graph.edges.iter().map(|e| (e.from_idx, e.to_idx)).collect();
    LayoutInput { nodes, edges, ticks: 1, alpha: graph.alpha, width, height }
}

// ── ResizeObserver setup ────────────────────────────────────────────────────

fn setup_resize_observer(
    canvas: &web_sys::HtmlCanvasElement,
    state: Rc<RefCell<RenderState>>,
    renderer: Rc<RefCell<Box<dyn Renderer>>>,
) {
    let canvas_clone = canvas.clone();
    let cb = Closure::<dyn FnMut(js_sys::Array)>::new(move |_entries: js_sys::Array| {
        let dpr = web_sys::window().unwrap().device_pixel_ratio();
        let css_w = canvas_clone.offset_width() as f64;
        let css_h = canvas_clone.offset_height() as f64;
        let pixel_w = (css_w * dpr) as u32;
        let pixel_h = (css_h * dpr) as u32;

        if canvas_clone.width() != pixel_w || canvas_clone.height() != pixel_h {
            canvas_clone.set_width(pixel_w);
            canvas_clone.set_height(pixel_h);
            renderer.borrow_mut().resize(pixel_w, pixel_h);
            let mut s = state.borrow_mut();
            s.viewport = Viewport::new(css_w, css_h);
        }
    });

    let observer = web_sys::ResizeObserver::new(cb.as_ref().unchecked_ref()).unwrap();
    observer.observe(canvas);
    // Leak both to keep alive for page lifetime
    std::mem::forget(cb);
    std::mem::forget(observer);
}

// ── RAF render loop ──────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn start_render_loop(
    state: Rc<RefCell<RenderState>>,
    renderer: Rc<RefCell<Box<dyn Renderer>>>,
    _bridge: PinnedBridge,
    _output_buf: Rc<RefCell<Option<LayoutOutput>>>,
    show_contradictions: RwSignal<bool>,
    show_bridges: RwSignal<bool>,
    simulation_running: RwSignal<bool>,
    zoom_in_count: RwSignal<u32>,
    zoom_out_count: RwSignal<u32>,
    temporal_min: RwSignal<u32>,
    temporal_max: RwSignal<u32>,
    visible_count: RwSignal<(usize, usize)>,
) -> RafHandle {
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();

    // Track previous zoom counts to detect button presses
    let prev_zoom_in = Rc::new(RefCell::new(0u32));
    let prev_zoom_out = Rc::new(RefCell::new(0u32));

    let closure_slot: ClosureSlot = Rc::new(RefCell::new(None));
    let closure_slot_inner = closure_slot.clone();

    let frame_closure = Closure::new(move || {
        if cancelled_clone.load(Ordering::Relaxed) {
            return;
        }

        let vis_count;

        {
            let mut s = state.borrow_mut();

            // Sync Leptos toggle signals into graph state
            s.graph.show_contradictions = show_contradictions.get_untracked();
            s.graph.show_bridges = show_bridges.get_untracked();
            let sim_running = simulation_running.get_untracked();

            // Handle zoom button presses
            let zi = zoom_in_count.get_untracked();
            let zo = zoom_out_count.get_untracked();
            let pzi = *prev_zoom_in.borrow();
            let pzo = *prev_zoom_out.borrow();
            if zi != pzi {
                *prev_zoom_in.borrow_mut() = zi;
                let cx = s.viewport.width() / 2.0;
                let cy = s.viewport.height() / 2.0;
                interaction::zoom_toward_cursor(&mut s.viewport, cx, cy, -1.0);
            }
            if zo != pzo {
                *prev_zoom_out.borrow_mut() = zo;
                let cx = s.viewport.width() / 2.0;
                let cy = s.viewport.height() / 2.0;
                interaction::zoom_toward_cursor(&mut s.viewport, cx, cy, 1.0);
            }

            // Run one force simulation tick inline (main thread).
            if sim_running && !s.graph.nodes.is_empty() && s.graph.alpha >= 0.001 {
                let canvas_w = s.viewport.width();
                let canvas_h = s.viewport.height();
                let input = build_layout_input(&s.graph, canvas_w, canvas_h);
                let output = resyn_worker::forces::run_ticks(&input);
                for (i, (x, y)) in output.positions.into_iter().enumerate() {
                    if i < s.graph.nodes.len() && !s.graph.nodes[i].pinned {
                        s.graph.nodes[i].x = x;
                        s.graph.nodes[i].y = y;
                    }
                }
                s.graph.velocities = output.velocities;
                s.graph.alpha = output.alpha;
                if output.converged {
                    s.graph.simulation_running = false;
                    simulation_running.set(false);
                }
            }

            // Snapshot viewport scale
            s.graph.current_scale = s.viewport.scale;
            // Extract values before mutable borrow of nodes (borrow checker requires this)
            let lod_scale = s.viewport.scale;
            let seed_id = s.graph.seed_paper_id.clone();
            // Update LOD visibility based on current zoom level
            crate::graph::lod::update_lod_visibility(
                &mut s.graph.nodes,
                lod_scale,
                &seed_id,
            );
            // Sync temporal range from signals
            s.graph.temporal_min_year = temporal_min.get_untracked();
            s.graph.temporal_max_year = temporal_max.get_untracked();
            let t_min = s.graph.temporal_min_year;
            let t_max = s.graph.temporal_max_year;
            // Update temporal visibility
            crate::graph::lod::update_temporal_visibility(
                &mut s.graph.nodes,
                t_min,
                t_max,
            );
            // Compute visible count (captured before render for signal update)
            vis_count = crate::graph::lod::compute_visible_count(&s.graph.nodes);

            // Render
            let graph = &s.graph;
            let viewport = &s.viewport;
            renderer.borrow_mut().draw(graph, viewport);
        }

        // Update visible count signal outside the state borrow (avoids RefCell conflicts)
        visible_count.set(vis_count);

        // Schedule next frame
        let window = web_sys::window().expect("no window");
        let slot = closure_slot_inner.borrow();
        if let Some(cb) = slot.as_ref() {
            window.request_animation_frame(cb.as_ref().unchecked_ref()).unwrap();
        }
    });

    *closure_slot.borrow_mut() = Some(frame_closure);

    let window = web_sys::window().expect("no window");
    {
        let slot = closure_slot.borrow();
        if let Some(cb) = slot.as_ref() {
            window.request_animation_frame(cb.as_ref().unchecked_ref()).unwrap();
        }
    }

    RafHandle { cancelled }
}

// ── Tooltip text formatting ─────────────────────────────────────────────────

fn node_tooltip(node: &crate::graph::layout_state::NodeState) -> String {
    let title = if node.title.len() > 60 {
        format!("{}…", &node.title[..60])
    } else {
        node.title.clone()
    };
    format!(
        "{}\n{} · {} · {} citations",
        title, node.first_author, node.year, node.citation_count
    )
}

fn edge_tooltip(graph: &GraphState, edge_idx: usize) -> String {
    use crate::server_fns::graph::EdgeType;
    let edge = &graph.edges[edge_idx];
    let from_node = &graph.nodes[edge.from_idx];
    let to_node = &graph.nodes[edge.to_idx];
    match edge.edge_type {
        EdgeType::Regular => {
            format!("{} cites {}", from_node.first_author, to_node.first_author)
        }
        EdgeType::Contradiction => {
            let confidence = edge.confidence.map(|c| (c * 100.0) as u32).unwrap_or(0);
            let terms: Vec<&str> =
                edge.shared_terms.iter().take(3).map(|s| s.as_str()).collect();
            format!("Contradiction · {}% · {}", confidence, terms.join(", "))
        }
        EdgeType::AbcBridge => {
            let confidence = edge.confidence.map(|c| (c * 100.0) as u32).unwrap_or(0);
            let terms: Vec<&str> =
                edge.shared_terms.iter().take(3).map(|s| s.as_str()).collect();
            format!("ABC-Bridge · {}% · {}", confidence, terms.join(", "))
        }
    }
}

// ── Event listener attachment ─────────────────────────────────────────────────

/// Get canvas-relative coordinates from a MouseEvent.
fn canvas_coords(canvas: &web_sys::HtmlCanvasElement, event: &web_sys::MouseEvent) -> (f64, f64) {
    let rect = canvas.get_bounding_client_rect();
    let sx = event.client_x() as f64 - rect.left();
    let sy = event.client_y() as f64 - rect.top();
    (sx, sy)
}

struct EventListeners {
    _mousemove: Closure<dyn FnMut(web_sys::MouseEvent)>,
    _mousedown: Closure<dyn FnMut(web_sys::MouseEvent)>,
    _mouseup: Closure<dyn FnMut(web_sys::MouseEvent)>,
    _dblclick: Closure<dyn FnMut(web_sys::MouseEvent)>,
    _wheel: Closure<dyn FnMut(web_sys::WheelEvent)>,
    _pointerleave: Closure<dyn FnMut(web_sys::PointerEvent)>,
}

#[allow(clippy::too_many_arguments)]
fn attach_event_listeners(
    canvas: &web_sys::HtmlCanvasElement,
    state: Rc<RefCell<RenderState>>,
    _bridge: PinnedBridge,
    tooltip_signal: RwSignal<Option<TooltipData>>,
    selected_paper: RwSignal<Option<DrawerOpenRequest>>,
) {
    let canvas_el = canvas.clone();

    // ── mousemove ────────────────────────────────────────────────────────────
    let state_mm = state.clone();
    let canvas_mm = canvas_el.clone();
    let mousemove =
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            let (sx, sy) = canvas_coords(&canvas_mm, &event);
            let mut s = state_mm.borrow_mut();
            let (wx, wy) = s.viewport.screen_to_world(sx, sy);

            match s.interaction.clone() {
                InteractionState::Panning {
                    start_x,
                    start_y,
                    start_offset_x,
                    start_offset_y,
                } => {
                    s.viewport.offset_x = start_offset_x + (sx - start_x);
                    s.viewport.offset_y = start_offset_y + (sy - start_y);
                    drop(s);
                    tooltip_signal.set(None);
                    return;
                }
                InteractionState::DraggingNode { node_idx, offset_x, offset_y } => {
                    if node_idx < s.graph.nodes.len() {
                        s.graph.nodes[node_idx].x = wx + offset_x;
                        s.graph.nodes[node_idx].y = wy + offset_y;
                    }
                    drop(s);
                    tooltip_signal.set(None);
                    return;
                }
                InteractionState::Idle => {}
            }

            // No active interaction — update hover state and tooltip
            let hovered = interaction::find_node_at(&s.graph.nodes, wx, wy);
            let hovered_edge = if hovered.is_none() {
                interaction::find_edge_at(&s.graph.nodes, &s.graph.edges, wx, wy, 4.0)
            } else {
                None
            };
            s.graph.hovered_node = hovered;
            s.graph.hovered_edge = hovered_edge;

            if let Some(idx) = hovered {
                let text = node_tooltip(&s.graph.nodes[idx]);
                drop(s);
                tooltip_signal.set(Some(TooltipData { text, x: sx, y: sy }));
            } else if let Some(eidx) = hovered_edge {
                let text = edge_tooltip(&s.graph, eidx);
                drop(s);
                tooltip_signal.set(Some(TooltipData { text, x: sx, y: sy }));
            } else {
                drop(s);
                tooltip_signal.set(None);
            }
        });

    canvas.add_event_listener_with_callback("mousemove", mousemove.as_ref().unchecked_ref())
        .unwrap();

    // ── mousedown ────────────────────────────────────────────────────────────
    let state_md = state.clone();
    let canvas_md = canvas_el.clone();
    let mousedown =
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            let (sx, sy) = canvas_coords(&canvas_md, &event);
            let mut s = state_md.borrow_mut();
            let (wx, wy) = s.viewport.screen_to_world(sx, sy);

            s.drag_start_x = sx;
            s.drag_start_y = sy;

            if let Some(node_idx) = interaction::find_node_at(&s.graph.nodes, wx, wy) {
                s.was_already_pinned = s.graph.nodes[node_idx].pinned;
                s.graph.nodes[node_idx].pinned = true;
                let node = &s.graph.nodes[node_idx];
                let offset_x = node.x - wx;
                let offset_y = node.y - wy;
                s.interaction =
                    InteractionState::DraggingNode { node_idx, offset_x, offset_y };
            } else {
                let start_offset_x = s.viewport.offset_x;
                let start_offset_y = s.viewport.offset_y;
                s.interaction = InteractionState::Panning {
                    start_x: sx,
                    start_y: sy,
                    start_offset_x,
                    start_offset_y,
                };
            }
        });

    canvas.add_event_listener_with_callback("mousedown", mousedown.as_ref().unchecked_ref())
        .unwrap();

    // ── mouseup ──────────────────────────────────────────────────────────────
    let state_mu = state.clone();
    let canvas_mu = canvas_el.clone();
    let mouseup =
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            let (sx, sy) = canvas_coords(&canvas_mu, &event);
            let mut s = state_mu.borrow_mut();

            let dx = sx - s.drag_start_x;
            let dy = sy - s.drag_start_y;
            let was_click = dx * dx + dy * dy < 9.0; // < 3px = click

            let prev_interaction = s.interaction.clone();
            s.interaction = InteractionState::Idle;

            match prev_interaction {
                InteractionState::DraggingNode { node_idx, .. } => {
                    if was_click {
                        // It was a click, not a drag
                        if s.was_already_pinned {
                            // Node was already pinned → unpin it
                            s.graph.nodes[node_idx].pinned = false;
                        } else {
                            // Node was NOT pinned → unpin (undo mousedown pin) and open drawer
                            s.graph.nodes[node_idx].pinned = false;
                            let paper_id = s.graph.nodes[node_idx].id.clone();
                            s.graph.selected_node = Some(node_idx);
                            drop(s);
                            selected_paper.set(Some(DrawerOpenRequest {
                                paper_id,
                                ..Default::default()
                            }));
                        }
                    }
                    // Real drag → node stays pinned (was set in mousedown)
                }
                InteractionState::Panning { .. } => {
                    if was_click {
                        // Click on background — deselect
                        s.graph.selected_node = None;
                        drop(s);
                        selected_paper.set(None);
                    }
                }
                InteractionState::Idle => {}
            }
        });

    canvas.add_event_listener_with_callback("mouseup", mouseup.as_ref().unchecked_ref())
        .unwrap();

    // ── dblclick — reset viewport ────────────────────────────────────────────
    let state_dc = state.clone();
    let canvas_dc = canvas_el.clone();
    let dblclick =
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |_event: web_sys::MouseEvent| {
            let mut s = state_dc.borrow_mut();
            let w = canvas_dc.offset_width() as f64;
            let h = canvas_dc.offset_height() as f64;
            s.viewport = Viewport::new(w, h);
        });

    canvas.add_event_listener_with_callback("dblclick", dblclick.as_ref().unchecked_ref())
        .unwrap();

    // ── wheel — zoom toward cursor (normalized delta) ────────────────────────
    let state_wh = state.clone();
    let canvas_wh = canvas_el.clone();
    let wheel =
        Closure::<dyn FnMut(web_sys::WheelEvent)>::new(move |event: web_sys::WheelEvent| {
            event.prevent_default();
            let rect = canvas_wh.get_bounding_client_rect();
            let cx = event.client_x() as f64 - rect.left();
            let cy = event.client_y() as f64 - rect.top();
            // Normalize: most browsers send ~100 for one wheel notch
            let raw_delta = event.delta_y();
            let normalized = if raw_delta.abs() > 50.0 {
                raw_delta.signum()
            } else {
                raw_delta / 50.0
            };
            let mut s = state_wh.borrow_mut();
            interaction::zoom_toward_cursor(&mut s.viewport, cx, cy, normalized);
        });

    canvas.add_event_listener_with_callback("wheel", wheel.as_ref().unchecked_ref()).unwrap();

    // ── pointerleave — clear hover state ────────────────────────────────────
    let state_pl = state.clone();
    let pointerleave =
        Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |_event: web_sys::PointerEvent| {
            let mut s = state_pl.borrow_mut();
            s.graph.hovered_node = None;
            s.graph.hovered_edge = None;
            s.interaction = InteractionState::Idle;
            drop(s);
            tooltip_signal.set(None);
        });

    canvas
        .add_event_listener_with_callback("pointerleave", pointerleave.as_ref().unchecked_ref())
        .unwrap();

    let _listeners = EventListeners {
        _mousemove: mousemove,
        _mousedown: mousedown,
        _mouseup: mouseup,
        _dblclick: dblclick,
        _wheel: wheel,
        _pointerleave: pointerleave,
    };
    std::mem::forget(_listeners);
}
