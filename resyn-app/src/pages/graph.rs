use leptos::html::Canvas;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::app::{DrawerOpenRequest, SearchPanTrigger, SelectedPaper};
use crate::graph::viewport_fit::compute_single_node_pan_target;
use crate::components::graph_controls::{GraphControls, TemporalSlider};
use crate::graph::interaction::{self, InteractionState};
use crate::graph::layout_state::GraphState;
use crate::graph::make_renderer;
use crate::graph::renderer::{Renderer, Viewport};
use crate::graph::worker_bridge::WorkerBridge;
use crate::server_fns::graph::get_graph_data;
use resyn_worker::{LayoutInput, LayoutOutput, NodeData};

use crate::graph::label_collision::draw_label_pill;
use crate::graph::layout_state::LabelMode;

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
    // Phase 17 — viewport fit
    fit_anim: crate::graph::viewport_fit::FitAnimState,
    user_has_interacted: bool,
    fit_has_fired_once: bool,
    label_cache_dirty: bool,
    /// Cached text widths per node (measured once at graph load via measureText).
    text_widths: Vec<f64>,
    /// Cached text widths for keyword mode (measured once at graph load).
    keyword_text_widths: Vec<f64>,
    /// Previous label mode — used to detect mode changes and invalidate caches.
    prev_label_mode: LabelMode,
    /// Cached label collision result (stored here so it works for both Canvas2D and WebGL2).
    label_cache: Option<crate::graph::label_collision::LabelCache>,
    /// Cached cluster result for cluster label rendering in Keywords mode.
    cluster_result: Option<crate::graph::kmeans::ClusterResult>,
    /// Frame counter for cluster recompute scheduling (every 10 frames during simulation).
    cluster_frame_counter: u32,
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
    let label_canvas_ref = NodeRef::<Canvas>::new();

    // Overlay toggle signals — Leptos reactive (drive UI buttons)
    let show_contradictions: RwSignal<bool> = RwSignal::new(true);
    let show_bridges: RwSignal<bool> = RwSignal::new(true);
    let simulation_running: RwSignal<bool> = RwSignal::new(true);

    // Topic ring signals
    let show_topic_rings: RwSignal<bool> = RwSignal::new(true); // D-09: default on
    let active_topic_filter: RwSignal<std::collections::HashSet<String>> =
        RwSignal::new(std::collections::HashSet::new());
    let palette_signal: RwSignal<Vec<crate::server_fns::graph::PaletteEntry>> =
        RwSignal::new(Vec::new());

    // Zoom button signals: increment a counter to trigger zoom from RAF loop
    let zoom_in_count: RwSignal<u32> = RwSignal::new(0);
    let zoom_out_count: RwSignal<u32> = RwSignal::new(0);

    // Fit button and simulation convergence signals
    let fit_count: RwSignal<u32> = RwSignal::new(0);
    let simulation_settled: RwSignal<bool> = RwSignal::new(false);

    // Temporal filter signals
    let temporal_min: RwSignal<u32> = RwSignal::new(2000);
    let temporal_max: RwSignal<u32> = RwSignal::new(2026);
    let year_bounds: RwSignal<(u32, u32)> = RwSignal::new((2000, 2026));
    let visible_count: RwSignal<(usize, usize)> = RwSignal::new((0, 0));

    // Label mode signal — controls which label style is rendered on graph nodes
    let label_mode: RwSignal<LabelMode> = RwSignal::new(LabelMode::AuthorYear);

    // Tooltip overlay signal
    let tooltip_signal: RwSignal<Option<TooltipData>> = RwSignal::new(None);

    // Fetch graph data from server
    let graph_resource = Resource::new(|| (), |_| get_graph_data());

    // SelectedPaper context — for drawer integration
    let SelectedPaper(selected_paper) = expect_context::<SelectedPaper>();

    // SearchPanTrigger context — for graph pan/highlight on search result selection
    let SearchPanTrigger(search_pan_signal) = expect_context::<SearchPanTrigger>();

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

        // Set up label overlay canvas (2D context for text, works with both Canvas2D and WebGL2)
        let label_ctx: Rc<RefCell<Option<web_sys::CanvasRenderingContext2d>>> =
            Rc::new(RefCell::new(None));
        if let Some(label_el) = label_canvas_ref.get() {
            let label_canvas: web_sys::HtmlCanvasElement = label_el;
            label_canvas.set_width((css_width * dpr) as u32);
            label_canvas.set_height((css_height * dpr) as u32);
            if let Ok(Some(ctx)) = label_canvas.get_context("2d") {
                let ctx: web_sys::CanvasRenderingContext2d = ctx.dyn_into().unwrap();
                *label_ctx.borrow_mut() = Some(ctx);
            }
        }

        let mut viewport = Viewport::new(css_width, css_height);
        let mut graph_state = GraphState::from_graph_data(data);
        // Populate palette_signal from graph_state (from_graph_data already moved data)
        palette_signal.set(graph_state.palette.clone());
        // Fit connected component into visible canvas. Only use nodes WITH
        // bfs_depth (connected nodes) — orphans at far outer ring would shrink
        // the interesting structure to a dot.
        if !graph_state.nodes.is_empty() {
            let connected_dists: Vec<f64> = graph_state
                .nodes
                .iter()
                .filter(|n| n.bfs_depth.is_some())
                .map(|n| (n.x * n.x + n.y * n.y).sqrt())
                .collect();
            let spread = if connected_dists.is_empty() {
                // Fallback: all nodes are orphans, use 90th percentile of all
                let mut dists: Vec<f64> = graph_state
                    .nodes
                    .iter()
                    .map(|n| (n.x * n.x + n.y * n.y).sqrt())
                    .collect();
                dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let idx = (dists.len() as f64 * 0.9) as usize;
                dists[idx.min(dists.len() - 1)]
            } else {
                *connected_dists
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap()
            }
            .max(1.0);
            let fit_scale = (css_width.min(css_height) * 0.4 / spread).min(1.0);
            viewport.scale = fit_scale;
        }
        let renderer = make_renderer(&canvas, graph_state.nodes.len());

        // Measure text widths for all node labels once at graph load time.
        // Uses a temporary 2D canvas context (measureText is a browser API that
        // works regardless of which renderer was selected for actual drawing).
        let (text_widths, keyword_text_widths) = {
            use wasm_bindgen::JsCast;
            let document = web_sys::window().unwrap().document().unwrap();
            let temp_canvas: web_sys::HtmlCanvasElement = document
                .create_element("canvas")
                .unwrap()
                .dyn_into()
                .unwrap();
            let temp_ctx = temp_canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();
            let author_year_widths = crate::graph::label_collision::build_text_widths(
                &temp_ctx,
                &graph_state.nodes,
                &LabelMode::AuthorYear,
            );
            let kw_widths = crate::graph::label_collision::build_text_widths(
                &temp_ctx,
                &graph_state.nodes,
                &LabelMode::Keywords,
            );
            (author_year_widths, kw_widths)
        };

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
            fit_anim: crate::graph::viewport_fit::FitAnimState::default(),
            user_has_interacted: false,
            fit_has_fired_once: false,
            label_cache_dirty: true,
            text_widths,
            keyword_text_widths,
            prev_label_mode: LabelMode::AuthorYear,
            label_cache: None,
            cluster_result: None,
            cluster_frame_counter: 0,
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
        setup_resize_observer(
            &canvas,
            state.clone(),
            renderer_rc.clone(),
            label_ctx.clone(),
        );

        // Attach event listeners to canvas
        attach_event_listeners(
            &canvas,
            state.clone(),
            pinned_bridge.clone(),
            tooltip_signal,
            selected_paper,
            simulation_running,
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
            fit_count,
            simulation_settled,
            label_ctx.clone(),
            label_mode,
            show_topic_rings,
            active_topic_filter,
            search_pan_signal,
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
                            <canvas
                                node_ref=label_canvas_ref
                                class="graph-canvas graph-label-overlay"
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
                                fit_count=fit_count
                                simulation_settled=simulation_settled
                                label_mode=label_mode
                                show_topic_rings=show_topic_rings
                                active_topic_filter=active_topic_filter
                                palette=palette_signal
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
        std::pin::Pin<Box<gloo_worker::reactor::ReactorBridge<resyn_worker::ForceLayoutWorker>>>,
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
            NodeData {
                x: n.x,
                y: n.y,
                vx,
                vy,
                mass: 1.0,
                pinned: n.pinned,
                radius: n.radius,
                bfs_depth: n.bfs_depth.unwrap_or(u32::MAX),
            }
        })
        .collect();
    let edges: Vec<(usize, usize)> = graph.edges.iter().map(|e| (e.from_idx, e.to_idx)).collect();
    LayoutInput {
        nodes,
        edges,
        ticks: 1,
        alpha: graph.alpha,
        width,
        height,
    }
}

// ── ResizeObserver setup ────────────────────────────────────────────────────

fn setup_resize_observer(
    canvas: &web_sys::HtmlCanvasElement,
    state: Rc<RefCell<RenderState>>,
    renderer: Rc<RefCell<Box<dyn Renderer>>>,
    label_ctx: Rc<RefCell<Option<web_sys::CanvasRenderingContext2d>>>,
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
            // Resize label overlay canvas to match
            if let Some(ref ctx) = *label_ctx.borrow()
                && let Some(c) = ctx.canvas()
            {
                c.set_width(pixel_w);
                c.set_height(pixel_h);
            }
            let mut s = state.borrow_mut();
            s.viewport = Viewport::new(css_w, css_h);
            s.label_cache_dirty = true;
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
    fit_count: RwSignal<u32>,
    simulation_settled: RwSignal<bool>,
    label_ctx: Rc<RefCell<Option<web_sys::CanvasRenderingContext2d>>>,
    label_mode: RwSignal<LabelMode>,
    show_topic_rings: RwSignal<bool>,
    active_topic_filter: RwSignal<std::collections::HashSet<String>>,
    search_pan_signal: RwSignal<Option<crate::app::SearchPanRequest>>,
) -> RafHandle {
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();

    // Track previous zoom/fit counts to detect button presses
    let prev_zoom_in = Rc::new(RefCell::new(0u32));
    let prev_zoom_out = Rc::new(RefCell::new(0u32));
    let prev_fit: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));

    // Track previous viewport for label cache dirty detection (D-12)
    let prev_scale = Rc::new(RefCell::new(1.0_f64));
    let prev_offset_x = Rc::new(RefCell::new(0.0_f64));
    let prev_offset_y = Rc::new(RefCell::new(0.0_f64));

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
                s.user_has_interacted = true;
                let cx = s.viewport.width() / 2.0;
                let cy = s.viewport.height() / 2.0;
                interaction::zoom_toward_cursor(&mut s.viewport, cx, cy, -1.0);
            }
            if zo != pzo {
                *prev_zoom_out.borrow_mut() = zo;
                s.user_has_interacted = true;
                let cx = s.viewport.width() / 2.0;
                let cy = s.viewport.height() / 2.0;
                interaction::zoom_toward_cursor(&mut s.viewport, cx, cy, 1.0);
            }

            // Handle fit button press
            let fi = fit_count.get_untracked();
            let pfi = *prev_fit.borrow();
            if fi != pfi {
                *prev_fit.borrow_mut() = fi;
                if let Some(fit) = crate::graph::viewport_fit::compute_fit_target(
                    &s.graph.nodes,
                    s.viewport.css_width,
                    s.viewport.css_height,
                ) {
                    s.fit_anim = fit;
                }
            }

            // Increment frame counter for pulse timing
            s.graph.frame_counter = s.graph.frame_counter.wrapping_add(1);

            // Check for pending search pan request
            if let Some(pan_req) = search_pan_signal.get_untracked() {
                let css_w = s.viewport.css_width;
                let css_h = s.viewport.css_height;
                let scale = s.viewport.scale;
                if let Some(fit) = compute_single_node_pan_target(
                    &s.graph.nodes,
                    &pan_req.paper_id,
                    css_w,
                    css_h,
                    scale,
                ) {
                    s.fit_anim = fit;
                    s.graph.search_highlighted = Some(pan_req.paper_id.clone());
                    s.graph.search_highlight_ids = vec![pan_req.paper_id.clone()];
                    s.graph.pulse_start_frame = None; // pulse starts after lerp completes
                    search_pan_signal.set(None); // consume the request
                }
            }

            // When fit animation just completed and search highlight is active, start pulse
            if !s.fit_anim.active
                && s.graph.search_highlighted.is_some()
                && s.graph.pulse_start_frame.is_none()
            {
                s.graph.pulse_start_frame = Some(s.graph.frame_counter);
            }

            // Check if pulse finished — clear search state
            if let Some(pulse_start) = s.graph.pulse_start_frame {
                let elapsed = s.graph.frame_counter.saturating_sub(pulse_start);
                if elapsed >= 120 {
                    s.graph.search_highlighted = None;
                    s.graph.pulse_start_frame = None;
                    s.graph.search_highlight_ids.clear();
                }
            }

            // Run one force simulation tick inline (main thread).
            // Simulation fully stops when alpha drops below ALPHA_MIN (D-09).
            // Drag reheat restarts it temporarily for local rearrangement (D-05).
            if sim_running && !s.graph.nodes.is_empty() {
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
                if s.graph.check_alpha_convergence() {
                    simulation_running.set(false);
                    simulation_settled.set(true);
                    if !s.user_has_interacted
                        && !s.fit_has_fired_once
                        && let Some(fit) = crate::graph::viewport_fit::compute_fit_target(
                            &s.graph.nodes,
                            s.viewport.css_width,
                            s.viewport.css_height,
                        )
                    {
                        s.fit_anim = fit;
                        s.fit_has_fired_once = true;
                    }
                }
            }

            // Animate viewport fit (D-01: ~0.5s lerp)
            if s.fit_anim.active {
                use crate::graph::viewport_fit::lerp;
                let t = 0.12; // exponential decay ease-out at 60fps
                s.viewport.scale = lerp(s.viewport.scale, s.fit_anim.target_scale, t);
                s.viewport.offset_x = lerp(s.viewport.offset_x, s.fit_anim.target_offset_x, t);
                s.viewport.offset_y = lerp(s.viewport.offset_y, s.fit_anim.target_offset_y, t);
                let close = (s.viewport.scale - s.fit_anim.target_scale).abs() < 0.001
                    && (s.viewport.offset_x - s.fit_anim.target_offset_x).abs() < 0.5
                    && (s.viewport.offset_y - s.fit_anim.target_offset_y).abs() < 0.5;
                if close {
                    s.viewport.scale = s.fit_anim.target_scale;
                    s.viewport.offset_x = s.fit_anim.target_offset_x;
                    s.viewport.offset_y = s.fit_anim.target_offset_y;
                    s.fit_anim.active = false;
                    s.label_cache_dirty = true;
                }
            }

            // Snapshot viewport scale
            s.graph.current_scale = s.viewport.scale;
            // Extract values before mutable borrow of nodes (borrow checker requires this)
            let lod_scale = s.viewport.scale;
            let seed_id = s.graph.seed_paper_id.clone();
            // Update LOD visibility based on current zoom level
            crate::graph::lod::update_lod_visibility(&mut s.graph.nodes, lod_scale, &seed_id);
            // Sync temporal range from signals
            s.graph.temporal_min_year = temporal_min.get_untracked();
            s.graph.temporal_max_year = temporal_max.get_untracked();
            let t_min = s.graph.temporal_min_year;
            let t_max = s.graph.temporal_max_year;
            // Update temporal visibility
            crate::graph::lod::update_temporal_visibility(&mut s.graph.nodes, t_min, t_max);
            // Compute visible count (captured before render for signal update)
            vis_count = crate::graph::lod::compute_visible_count(&s.graph.nodes);

            // Update topic_dimmed per-frame from active_topic_filter signal
            let topic_filter = active_topic_filter.get_untracked();
            if !topic_filter.is_empty() {
                for node in &mut s.graph.nodes {
                    node.topic_dimmed =
                        !node.top_keywords.iter().any(|(kw, _)| topic_filter.contains(kw));
                }
            } else {
                for node in &mut s.graph.nodes {
                    node.topic_dimmed = false;
                }
            }

            // Detect viewport changes for label cache dirty flag (D-12)
            let vp_changed = (s.viewport.scale - *prev_scale.borrow()).abs() > 0.0001
                || (s.viewport.offset_x - *prev_offset_x.borrow()).abs() > 0.1
                || (s.viewport.offset_y - *prev_offset_y.borrow()).abs() > 0.1;
            if vp_changed {
                s.label_cache_dirty = true;
            }
            *prev_scale.borrow_mut() = s.viewport.scale;
            *prev_offset_x.borrow_mut() = s.viewport.offset_x;
            *prev_offset_y.borrow_mut() = s.viewport.offset_y;

            // Cluster recompute (D-09): every 10 frames during simulation, or on first settle
            let just_converged = !sim_running && s.graph.alpha <= resyn_worker::forces::ALPHA_MIN;
            let current_label_mode_for_cluster = label_mode.get_untracked();
            if current_label_mode_for_cluster == LabelMode::Keywords {
                s.cluster_frame_counter = s.cluster_frame_counter.wrapping_add(1);
                let should_recompute = s.cluster_result.is_none()
                    || (sim_running && s.cluster_frame_counter % 10 == 0)
                    || just_converged;
                if should_recompute && s.graph.alpha < 0.15 {
                    // D-07: defer until simulation has partially settled (alpha < 0.15)
                    let positions: Vec<(f64, f64)> = s
                        .graph
                        .nodes
                        .iter()
                        .filter(|n| n.lod_visible && n.temporal_visible)
                        .map(|n| (n.x, n.y))
                        .collect();
                    let keywords: Vec<Vec<(String, f32)>> = s
                        .graph
                        .nodes
                        .iter()
                        .filter(|n| n.lod_visible && n.temporal_visible)
                        .map(|n| n.top_keywords.clone())
                        .collect();
                    if positions.len() >= 3 {
                        s.cluster_result =
                            Some(crate::graph::kmeans::compute_clusters(&positions, &keywords));
                    }
                }
            } else {
                s.cluster_result = None;
            }

            // Detect label mode changes and invalidate cache
            let current_label_mode = label_mode.get_untracked();
            s.graph.label_mode = current_label_mode;
            if current_label_mode != s.prev_label_mode {
                s.label_cache_dirty = true;
                s.prev_label_mode = current_label_mode;
            }

            // Rebuild label cache when dirty and not animating
            if s.fit_anim.active {
                s.label_cache = None;
            } else if s.label_cache_dirty {
                let widths = match s.graph.label_mode {
                    LabelMode::Keywords => &s.keyword_text_widths,
                    _ => &s.text_widths,
                };
                let cache = crate::graph::label_collision::build_label_cache(
                    &s.graph.nodes,
                    widths,
                    &s.viewport,
                );
                s.label_cache = Some(cache);
                s.label_cache_dirty = false;
            }

            // Render main scene
            let graph = &s.graph;
            let viewport = &s.viewport;
            renderer.borrow_mut().draw(graph, viewport);

            // Draw labels on the overlay canvas (works for both Canvas2D and WebGL2)
            if let Some(ref ctx) = *label_ctx.borrow() {
                let dpr = web_sys::window().unwrap().device_pixel_ratio();
                let cw = s.viewport.css_width * dpr;
                let ch = s.viewport.css_height * dpr;
                ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
                ctx.clear_rect(0.0, 0.0, cw, ch);
                ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0).unwrap();
                ctx.set_global_alpha(1.0);

                // Similarity edges on label canvas overlay — drawn before topic rings so rings
                // appear on top. This is the rendering path for WebGL2 mode (Canvas2D has its
                // own draw pass in canvas_renderer.rs step 6.5). Drawing here also means both
                // renderers share one label canvas path for similarity edges.
                if s.graph.show_similarity {
                    use crate::server_fns::graph::EdgeType;
                    use js_sys::Array;
                    use wasm_bindgen::JsValue;
                    ctx.save();
                    ctx.set_global_alpha(0.7);
                    let dash_array = Array::new();
                    dash_array.push(&JsValue::from_f64(8.0));
                    dash_array.push(&JsValue::from_f64(5.0));
                    ctx.set_line_dash(&dash_array).unwrap();
                    for edge in &s.graph.edges {
                        if edge.edge_type != EdgeType::Similarity {
                            continue;
                        }
                        let from = match s.graph.nodes.get(edge.from_idx) {
                            Some(n) => n,
                            None => continue,
                        };
                        let to = match s.graph.nodes.get(edge.to_idx) {
                            Some(n) => n,
                            None => continue,
                        };
                        let (fx, fy) = s.viewport.world_to_screen(from.x, from.y);
                        let (tx, ty) = s.viewport.world_to_screen(to.x, to.y);
                        let score = edge.confidence.unwrap_or(0.3);
                        let thickness = 1.5 + score as f64 * 2.5;
                        ctx.set_stroke_style_str("#f0a030");
                        ctx.set_line_width(thickness);
                        ctx.begin_path();
                        ctx.move_to(fx, fy);
                        ctx.line_to(tx, ty);
                        ctx.stroke();
                    }
                    ctx.restore();
                }

                // Topic rings — drawn at all zoom levels; individual node threshold handles
                // visibility via MIN_SCREEN_RADIUS_FOR_RINGS
                if show_topic_rings.get_untracked() {
                    draw_topic_rings(
                        ctx,
                        &s.graph.nodes,
                        &s.graph.palette,
                        &s.viewport,
                        &s.graph.seed_paper_id,
                        s.graph.hovered_node,
                    );
                }

                if !s.fit_anim.active && s.viewport.scale > 0.3 {
                    ctx.set_font("11px monospace");

                    use crate::graph::label_collision::{
                        LABEL_NODE_GAP, PILL_CORNER_RADIUS, PILL_H_PAD, PILL_HEIGHT,
                        draw_keyword_pills, draw_not_analyzed_badge,
                    };

                    match s.graph.label_mode {
                        LabelMode::Off => {
                            // Still show hover label in Off mode
                            if let Some(hi) = s.graph.hovered_node {
                                if hi < s.graph.nodes.len() {
                                    let node = &s.graph.nodes[hi];
                                    if node.lod_visible && node.temporal_visible {
                                        let (sx, sy) =
                                            s.viewport.world_to_screen(node.x, node.y);
                                        let text_w =
                                            s.text_widths.get(hi).copied().unwrap_or(40.0);
                                        let pill_w = text_w + PILL_H_PAD * 2.0;
                                        let label_x = sx - pill_w / 2.0;
                                        let label_y = sy
                                            + node.radius * s.viewport.scale
                                            + LABEL_NODE_GAP;
                                        draw_label_pill(
                                            ctx,
                                            label_x,
                                            label_y,
                                            pill_w,
                                            PILL_HEIGHT,
                                            PILL_CORNER_RADIUS,
                                            &node.label(),
                                            PILL_H_PAD,
                                            1.0,
                                        );
                                    }
                                }
                            }
                        }
                        LabelMode::AuthorYear => {
                            // Draw collision-culled labels from cache
                            if let Some(ref cache) = s.label_cache {
                                for &i in &cache.visible_indices {
                                    let node = &s.graph.nodes[i];
                                    if !node.lod_visible || !node.temporal_visible {
                                        continue;
                                    }
                                    let (sx, sy) = s.viewport.world_to_screen(node.x, node.y);
                                    let text_w = s.text_widths.get(i).copied().unwrap_or(40.0);
                                    let pill_w = text_w + PILL_H_PAD * 2.0;
                                    let label_x = sx - pill_w / 2.0;
                                    let label_y =
                                        sy + node.radius * s.viewport.scale + LABEL_NODE_GAP;
                                    draw_label_pill(
                                        ctx,
                                        label_x,
                                        label_y,
                                        pill_w,
                                        PILL_HEIGHT,
                                        PILL_CORNER_RADIUS,
                                        &node.label(),
                                        PILL_H_PAD,
                                        1.0,
                                    );
                                }
                            }
                            // Also show hover label if hovered node not in cache
                            if let Some(hi) = s.graph.hovered_node {
                                if hi < s.graph.nodes.len() {
                                    let in_cache = s.label_cache.as_ref().map_or(false, |c| {
                                        c.visible_indices.contains(&hi)
                                    });
                                    if !in_cache {
                                        let node = &s.graph.nodes[hi];
                                        if node.lod_visible && node.temporal_visible {
                                            let (sx, sy) =
                                                s.viewport.world_to_screen(node.x, node.y);
                                            let text_w =
                                                s.text_widths.get(hi).copied().unwrap_or(40.0);
                                            let pill_w = text_w + PILL_H_PAD * 2.0;
                                            let label_x = sx - pill_w / 2.0;
                                            let label_y = sy
                                                + node.radius * s.viewport.scale
                                                + LABEL_NODE_GAP;
                                            draw_label_pill(
                                                ctx,
                                                label_x,
                                                label_y,
                                                pill_w,
                                                PILL_HEIGHT,
                                                PILL_CORNER_RADIUS,
                                                &node.label(),
                                                PILL_H_PAD,
                                                1.0,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        LabelMode::Keywords => {
                            // Draw collision-culled keyword pills from cache
                            if let Some(ref cache) = s.label_cache {
                                for &i in &cache.visible_indices {
                                    let node = &s.graph.nodes[i];
                                    if !node.lod_visible || !node.temporal_visible {
                                        continue;
                                    }
                                    let (sx, sy) = s.viewport.world_to_screen(node.x, node.y);
                                    let label_y =
                                        sy + node.radius * s.viewport.scale + LABEL_NODE_GAP;
                                    if node.top_keywords.is_empty() {
                                        // D-06: unanalyzed badge
                                        let text_w = s
                                            .keyword_text_widths
                                            .get(i)
                                            .copied()
                                            .unwrap_or(80.0);
                                        draw_not_analyzed_badge(ctx, sx, label_y, text_w);
                                    } else {
                                        // D-02: top-2 keyword pills
                                        let top2: Vec<(String, f32)> = node
                                            .top_keywords
                                            .iter()
                                            .take(2)
                                            .cloned()
                                            .collect();
                                        let pill_widths: Vec<f64> = top2
                                            .iter()
                                            .map(|(term, _)| {
                                                ctx.measure_text(term).unwrap().width()
                                            })
                                            .collect();
                                        draw_keyword_pills(
                                            ctx,
                                            sx,
                                            label_y,
                                            &top2,
                                            &pill_widths,
                                        );
                                    }
                                }
                            }
                            // Draw cluster labels when zoomed out (D-08, D-10)
                            if s.viewport.scale < crate::graph::lod::LOD_LEVEL_1
                                && let Some(ref clusters) = s.cluster_result
                            {
                                crate::graph::label_collision::draw_cluster_labels(
                                    ctx,
                                    clusters,
                                    &s.graph.nodes,
                                    &s.viewport,
                                );
                            }
                        }
                    }
                }
            }
        } // end state borrow scope

        // Update visible count signal outside the state borrow (avoids RefCell conflicts)
        visible_count.set(vis_count);

        // Schedule next frame
        let window = web_sys::window().expect("no window");
        let slot = closure_slot_inner.borrow();
        if let Some(cb) = slot.as_ref() {
            window
                .request_animation_frame(cb.as_ref().unchecked_ref())
                .unwrap();
        }
    });

    *closure_slot.borrow_mut() = Some(frame_closure);

    let window = web_sys::window().expect("no window");
    {
        let slot = closure_slot.borrow();
        if let Some(cb) = slot.as_ref() {
            window
                .request_animation_frame(cb.as_ref().unchecked_ref())
                .unwrap();
        }
    }

    RafHandle { cancelled }
}

// ── Tooltip text formatting ─────────────────────────────────────────────────

fn node_tooltip(
    node: &crate::graph::layout_state::NodeState,
    label_mode: &crate::graph::layout_state::LabelMode,
) -> String {
    let title = if node.title.len() > 60 {
        format!("{}…", &node.title[..60])
    } else {
        node.title.clone()
    };
    let mut text = format!(
        "{}\n{} · {} · {} citations",
        title, node.first_author, node.year, node.citation_count
    );

    // D-05: Append keywords section when in keyword mode and keywords present
    if *label_mode == crate::graph::layout_state::LabelMode::Keywords
        && !node.top_keywords.is_empty()
    {
        text.push_str("\n────────────────────\nKeywords:");
        for (term, score) in node.top_keywords.iter().take(5) {
            text.push_str(&format!("\n  {}  {:.2}", term, score));
        }
    }
    text
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
            let terms: Vec<&str> = edge
                .shared_terms
                .iter()
                .take(3)
                .map(|s| s.as_str())
                .collect();
            format!("Contradiction · {}% · {}", confidence, terms.join(", "))
        }
        EdgeType::AbcBridge => {
            let confidence = edge.confidence.map(|c| (c * 100.0) as u32).unwrap_or(0);
            let terms: Vec<&str> = edge
                .shared_terms
                .iter()
                .take(3)
                .map(|s| s.as_str())
                .collect();
            format!("ABC-Bridge · {}% · {}", confidence, terms.join(", "))
        }
        EdgeType::Similarity => {
            let score = edge.confidence.map(|c| (c * 100.0) as u32).unwrap_or(0);
            let terms: Vec<&str> = edge
                .shared_terms
                .iter()
                .take(3)
                .map(|s| s.as_str())
                .collect();
            if terms.is_empty() {
                format!(
                    "Similar · {}% · {} ↔ {}",
                    score, from_node.first_author, to_node.first_author
                )
            } else {
                format!("Similar · {}% · {}", score, terms.join(", "))
            }
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
    simulation_running: RwSignal<bool>,
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
                    s.user_has_interacted = true;
                    s.viewport.offset_x = start_offset_x + (sx - start_x);
                    s.viewport.offset_y = start_offset_y + (sy - start_y);
                    drop(s);
                    tooltip_signal.set(None);
                    return;
                }
                InteractionState::DraggingNode {
                    node_idx,
                    offset_x,
                    offset_y,
                } => {
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
                let text = node_tooltip(&s.graph.nodes[idx], &s.graph.label_mode);
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

    canvas
        .add_event_listener_with_callback("mousemove", mousemove.as_ref().unchecked_ref())
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
                s.interaction = InteractionState::DraggingNode {
                    node_idx,
                    offset_x,
                    offset_y,
                };
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

    canvas
        .add_event_listener_with_callback("mousedown", mousedown.as_ref().unchecked_ref())
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

            let mut reheat_simulation = false;
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
                            simulation_running.set(true);
                            return;
                        }
                    } else {
                        // Real drag → unpin node so it settles naturally with neighbors.
                        // Only keep pinned if it was already pinned before drag started.
                        if !s.was_already_pinned {
                            s.graph.nodes[node_idx].pinned = false;
                        }
                    }
                    // Gentle local reheat — just enough for neighbors to adjust.
                    s.graph.alpha = 0.02_f64.max(s.graph.alpha);
                    s.graph.simulation_running = true;
                    reheat_simulation = true;
                }
                InteractionState::Panning { .. } => {
                    if was_click {
                        // Click on background — deselect
                        s.graph.selected_node = None;
                        drop(s);
                        selected_paper.set(None);
                        return;
                    }
                }
                InteractionState::Idle => {}
            }
            drop(s);
            if reheat_simulation {
                simulation_running.set(true);
            }
        });

    canvas
        .add_event_listener_with_callback("mouseup", mouseup.as_ref().unchecked_ref())
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

    canvas
        .add_event_listener_with_callback("dblclick", dblclick.as_ref().unchecked_ref())
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
            s.user_has_interacted = true;
            interaction::zoom_toward_cursor(&mut s.viewport, cx, cy, normalized);
        });

    canvas
        .add_event_listener_with_callback("wheel", wheel.as_ref().unchecked_ref())
        .unwrap();

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

// ── Topic ring rendering ──────────────────────────────────────────────────────

const MIN_SCREEN_RADIUS_FOR_RINGS: f64 = 14.0; // only show when arcs are distinguishable
const TOPIC_RING_WIDTH_FRAC: f64 = 0.12; // ring width as fraction of screen radius
const TOPIC_RING_MIN_WIDTH: f64 = 1.5;
const TOPIC_RING_MAX_WIDTH: f64 = 5.0;
const TOPIC_RING_HOVER_EXTRA: f64 = 2.5; // extra px added on hover
const START_ANGLE: f64 = -std::f64::consts::FRAC_PI_2; // -PI/2 = 12 o'clock

/// Compute arc segment angles for a node's top-3 keywords.
/// Returns Vec of (start_angle, end_angle, slot_index_or_None).
/// None in the third position means "neutral remainder" or keyword not in palette.
///
/// Each keyword score is treated as a fraction of the full circle (2π).
/// If the total of top-3 scores is less than 1.0, the remaining circumference
/// is appended as a neutral arc (D-07). Scores above 1.0 are clamped to 1.0.
fn compute_arc_angles(
    top_keywords: &[(String, f32)],
    palette_slot_map: &std::collections::HashMap<&str, u8>,
) -> Vec<(f64, f64, Option<u8>)> {
    use std::f64::consts::TAU;

    if top_keywords.is_empty() {
        return vec![];
    }

    let top3: Vec<&(String, f32)> = top_keywords.iter().take(3).collect();

    let mut arcs = Vec::new();
    let mut current_angle = START_ANGLE;

    for (keyword, score) in &top3 {
        // Each score is a fraction of the full circle; clamp individual scores to [0, 1]
        let clamped = score.clamp(0.0, 1.0);
        let arc_angle = TAU * clamped as f64;
        let end_angle = current_angle + arc_angle;
        let slot = palette_slot_map.get(keyword.as_str()).copied();
        arcs.push((current_angle, end_angle, slot));
        current_angle = end_angle;
    }

    // Neutral remainder arc if scores sum < 1.0 (circumference not fully covered)
    let end = START_ANGLE + TAU;
    if current_angle < end - 0.01 {
        arcs.push((current_angle, end, None));
    }

    arcs
}

fn draw_topic_rings(
    ctx: &web_sys::CanvasRenderingContext2d,
    nodes: &[crate::graph::layout_state::NodeState],
    palette: &[crate::server_fns::graph::PaletteEntry],
    viewport: &crate::graph::renderer::Viewport,
    _seed_paper_id: &Option<String>,
    hovered_node: Option<usize>,
) {
    // Build keyword -> color_hex lookup from palette
    let palette_color_map: std::collections::HashMap<&str, String> = palette
        .iter()
        .map(|e| {
            (
                e.keyword.as_str(),
                format!("#{:02x}{:02x}{:02x}", e.r, e.g, e.b),
            )
        })
        .collect();
    let palette_slot_map: std::collections::HashMap<&str, u8> = palette
        .iter()
        .map(|e| (e.keyword.as_str(), e.slot_index))
        .collect();

    // Pre-build slot_index -> color_hex for arc rendering
    let slot_colors: std::collections::HashMap<u8, String> = palette
        .iter()
        .map(|e| {
            let color = palette_color_map
                .get(e.keyword.as_str())
                .cloned()
                .unwrap_or_else(|| "#6e7681".to_string());
            (e.slot_index, color)
        })
        .collect();

    for (i, node) in nodes.iter().enumerate() {
        if !node.lod_visible || !node.temporal_visible {
            continue;
        }

        let (sx, sy) = viewport.world_to_screen(node.x, node.y);
        let screen_radius = node.radius * viewport.scale;

        // D-13: Below threshold, skip topic rings (existing border handles it)
        if screen_radius < MIN_SCREEN_RADIUS_FOR_RINGS {
            continue;
        }

        let dim_alpha = if node.topic_dimmed { 0.3 } else { 1.0 };

        // Scale-aware ring width: proportional to screen radius, clamped
        let is_hovered = hovered_node == Some(i);
        let base_width = (screen_radius * TOPIC_RING_WIDTH_FRAC)
            .clamp(TOPIC_RING_MIN_WIDTH, TOPIC_RING_MAX_WIDTH);
        let ring_width = if is_hovered { base_width + TOPIC_RING_HOVER_EXTRA } else { base_width };

        if node.top_keywords.is_empty() {
            // D-15: Dashed ring for unanalyzed nodes
            draw_dashed_ring(ctx, sx, sy, screen_radius, dim_alpha, ring_width);
            continue;
        }

        let arcs = compute_arc_angles(&node.top_keywords, &palette_slot_map);

        for (start, end, slot) in &arcs {
            let color = match slot {
                Some(idx) => slot_colors
                    .get(idx)
                    .map(|s| s.as_str())
                    .unwrap_or("#6e7681"),
                None => "#3a424c", // neutral remainder (D-07)
            };

            ctx.save();
            ctx.set_global_alpha(dim_alpha);
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(ring_width);
            ctx.begin_path();
            let _ = ctx.arc(sx, sy, screen_radius, *start, *end);
            ctx.stroke();
            ctx.restore();
        }
    }
}

fn draw_dashed_ring(
    ctx: &web_sys::CanvasRenderingContext2d,
    sx: f64,
    sy: f64,
    r: f64,
    alpha: f64,
    width: f64,
) {
    use std::f64::consts::TAU;

    let dash_array = js_sys::Array::new();
    dash_array.push(&wasm_bindgen::JsValue::from_f64(4.0));
    dash_array.push(&wasm_bindgen::JsValue::from_f64(4.0));

    ctx.save();
    ctx.set_global_alpha(alpha);
    let _ = ctx.set_line_dash(&dash_array);
    ctx.set_stroke_style_str("#6e7681");
    ctx.set_line_width(width);
    ctx.begin_path();
    let _ = ctx.arc(sx, sy, r, 0.0, TAU);
    ctx.stroke();
    // Reset dash pattern before restore (avoid leaking into subsequent draws)
    let _ = ctx.set_line_dash(&js_sys::Array::new());
    ctx.restore();
}

#[cfg(test)]
mod topic_ring_tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, TAU};

    #[test]
    fn test_arc_angles_empty_keywords() {
        let palette = std::collections::HashMap::new();
        let arcs = compute_arc_angles(&[], &palette);
        assert!(arcs.is_empty());
    }

    #[test]
    fn test_arc_angles_single_keyword_full_circle() {
        let mut palette = std::collections::HashMap::new();
        palette.insert("quantum", 0u8);
        let keywords = vec![("quantum".to_string(), 1.0f32)];
        let arcs = compute_arc_angles(&keywords, &palette);
        // Single keyword with score 1.0 should fill entire circle, no remainder
        assert_eq!(arcs.len(), 1);
        assert_eq!(arcs[0].2, Some(0)); // slot 0
        let arc_span = arcs[0].1 - arcs[0].0;
        assert!((arc_span - TAU).abs() < 0.01, "Expected full circle, got {arc_span}");
    }

    #[test]
    fn test_arc_angles_two_keywords_with_remainder() {
        let mut palette = std::collections::HashMap::new();
        palette.insert("quantum", 0u8);
        palette.insert("gravity", 1u8);
        let keywords = vec![
            ("quantum".to_string(), 0.3f32),
            ("gravity".to_string(), 0.2f32),
        ];
        let arcs = compute_arc_angles(&keywords, &palette);
        // Two keyword arcs + one neutral remainder = 3
        assert_eq!(arcs.len(), 3);
        assert_eq!(arcs[0].2, Some(0)); // quantum = slot 0
        assert_eq!(arcs[1].2, Some(1)); // gravity = slot 1
        assert_eq!(arcs[2].2, None); // neutral remainder
    }

    #[test]
    fn test_arc_angles_keyword_not_in_palette() {
        let palette = std::collections::HashMap::new(); // empty palette
        let keywords = vec![("unknown".to_string(), 0.5f32)];
        let arcs = compute_arc_angles(&keywords, &palette);
        assert!(!arcs.is_empty());
        assert_eq!(arcs[0].2, None); // not in palette -> None
    }

    #[test]
    fn test_arc_angles_start_at_12_oclock() {
        let palette = std::collections::HashMap::new();
        let keywords = vec![("kw".to_string(), 0.5f32)];
        let arcs = compute_arc_angles(&keywords, &palette);
        assert!(
            (arcs[0].0 - (-FRAC_PI_2)).abs() < 0.001,
            "Should start at -PI/2 (12 o'clock)"
        );
    }

    #[test]
    fn test_arc_angles_takes_only_top_3() {
        let palette = std::collections::HashMap::new();
        let keywords = vec![
            ("a".to_string(), 0.25f32),
            ("b".to_string(), 0.25f32),
            ("c".to_string(), 0.25f32),
            ("d".to_string(), 0.25f32), // 4th keyword should be ignored
        ];
        let arcs = compute_arc_angles(&keywords, &palette);
        // 3 keyword arcs + 1 remainder (since 0.75 < 1.0) = 4 arcs total
        assert!(arcs.len() <= 4);
    }
}
