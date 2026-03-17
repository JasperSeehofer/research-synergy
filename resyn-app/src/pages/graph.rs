use leptos::html::Canvas;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::app::SelectedPaper;
use crate::components::graph_controls::GraphControls;
use crate::graph::canvas_renderer::Canvas2DRenderer;
use crate::graph::interaction::{self, InteractionState};
use crate::graph::layout_state::GraphState;
use crate::graph::renderer::{Renderer, Viewport};
use crate::graph::worker_bridge::WorkerBridge;
use crate::server_fns::graph::get_graph_data;
use futures::Stream;
use resyn_worker::{LayoutInput, LayoutOutput, NodeData};

// ── Tooltip data ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct TooltipData {
    pub text: String,
    pub x: f64,
    pub y: f64,
}

// ── Shared render state (outside Leptos reactive graph) ─────────────────────
// Renderer is kept separate (in renderer_rc) so we can mutably borrow it
// while immutably borrowing GraphState and Viewport in the same frame.

struct RenderState {
    graph: GraphState,
    viewport: Viewport,
    interaction: InteractionState,
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
        let canvas: web_sys::HtmlCanvasElement = canvas_el.into();
        let Some(Ok(data)) = graph_resource.get() else {
            return;
        };

        // Size canvas to its CSS container
        let width = canvas.offset_width() as u32;
        let height = canvas.offset_height() as u32;
        canvas.set_width(width);
        canvas.set_height(height);

        let renderer = Canvas2DRenderer::new(&canvas);
        let viewport = Viewport::new(width as f64, height as f64);
        let mut graph_state = GraphState::from_graph_data(data);
        // Sync initial toggle values from Leptos signals
        graph_state.show_contradictions = show_contradictions.get_untracked();
        graph_state.show_bridges = show_bridges.get_untracked();
        graph_state.simulation_running = simulation_running.get_untracked();

        let state = Rc::new(RefCell::new(RenderState {
            graph: graph_state,
            viewport,
            interaction: InteractionState::Idle,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
        }));

        // Renderer is stored separately from RenderState to avoid borrow conflicts
        // when calling draw(&graph, &viewport) while mutably borrowing renderer.
        let renderer_rc: Rc<RefCell<Box<dyn Renderer>>> =
            Rc::new(RefCell::new(Box::new(renderer)));

        // Create worker bridge and pin it for Stream polling in the RAF loop.
        // ReactorBridge<ForceLayoutWorker> implements Stream<Item = LayoutOutput>.
        // We pin it in a Box so we can poll it with a noop waker each frame.
        let bridge = WorkerBridge::new();
        let pinned_bridge: Rc<RefCell<std::pin::Pin<Box<gloo_worker::reactor::ReactorBridge<resyn_worker::ForceLayoutWorker>>>>> =
            Rc::new(RefCell::new(Box::pin(bridge.bridge)));

        // Send initial layout request
        {
            let s = state.borrow();
            let input = build_layout_input(&s.graph, width as f64, height as f64);
            // Use send_input directly on the pinned bridge (it takes &self)
            pinned_bridge.borrow().as_ref().get_ref().send_input(input);
        }

        // Attach event listeners to canvas
        attach_event_listeners(
            &canvas,
            state.clone(),
            pinned_bridge.clone(),
            tooltip_signal,
            selected_paper,
            width as f64,
            height as f64,
        );

        // Start RAF render loop
        let handle = start_render_loop(
            state.clone(),
            renderer_rc.clone(),
            pinned_bridge.clone(),
            show_contradictions,
            show_bridges,
            simulation_running,
            width as f64,
            height as f64,
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

// ── Pinned bridge type alias ─────────────────────────────────────────────────

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
        .map(|n| NodeData { x: n.x, y: n.y, mass: 1.0, pinned: n.pinned })
        .collect();
    let edges: Vec<(usize, usize)> =
        graph.edges.iter().map(|e| (e.from_idx, e.to_idx)).collect();
    LayoutInput { nodes, edges, ticks: 1, width, height }
}

/// Poll the bridge synchronously using a noop waker — drains any ready outputs.
/// Returns the last received LayoutOutput if any were available.
fn poll_bridge_sync(bridge: &mut std::pin::Pin<Box<gloo_worker::reactor::ReactorBridge<resyn_worker::ForceLayoutWorker>>>) -> Option<LayoutOutput> {
    let waker = futures::task::noop_waker_ref();
    let mut cx = Context::from_waker(waker);
    let mut last = None;
    loop {
        match bridge.as_mut().poll_next(&mut cx) {
            Poll::Ready(Some(output)) => {
                last = Some(output);
            }
            Poll::Ready(None) | Poll::Pending => break,
        }
    }
    last
}

// ── RAF render loop ──────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn start_render_loop(
    state: Rc<RefCell<RenderState>>,
    renderer: Rc<RefCell<Box<dyn Renderer>>>,
    bridge: PinnedBridge,
    show_contradictions: RwSignal<bool>,
    show_bridges: RwSignal<bool>,
    simulation_running: RwSignal<bool>,
    canvas_width: f64,
    canvas_height: f64,
) -> RafHandle {
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();

    // Shared slot for the closure — required for self-referential RAF scheduling
    let closure_slot: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let closure_slot_inner = closure_slot.clone();

    let frame_closure = Closure::new(move || {
        if cancelled_clone.load(Ordering::Relaxed) {
            return;
        }

        {
            let mut s = state.borrow_mut();

            // Sync Leptos toggle signals into graph state (untracked to avoid subscriptions)
            s.graph.show_contradictions = show_contradictions.get_untracked();
            s.graph.show_bridges = show_bridges.get_untracked();
            let sim_running = simulation_running.get_untracked();

            // Drain any layout outputs from the worker bridge
            if let Some(output) = poll_bridge_sync(&mut bridge.borrow_mut()) {
                for (i, (x, y)) in output.positions.into_iter().enumerate() {
                    if i < s.graph.nodes.len() && !s.graph.nodes[i].pinned {
                        s.graph.nodes[i].x = x;
                        s.graph.nodes[i].y = y;
                    }
                }
                if output.converged {
                    s.graph.simulation_running = false;
                }
            }

            // Send next layout tick to worker if simulation is running
            if sim_running && !s.graph.nodes.is_empty() {
                let input = build_layout_input(&s.graph, canvas_width, canvas_height);
                bridge.borrow().as_ref().get_ref().send_input(input);
            }

            // Render this frame: renderer is borrowed separately to avoid
            // conflict with the immutable borrows of graph and viewport
            let graph = &s.graph;
            let viewport = &s.viewport;
            renderer.borrow_mut().draw(graph, viewport);
        }

        // Schedule next frame
        let window = web_sys::window().expect("no window");
        let slot = closure_slot_inner.borrow();
        if let Some(cb) = slot.as_ref() {
            window.request_animation_frame(cb.as_ref().unchecked_ref()).unwrap();
        }
    });

    *closure_slot.borrow_mut() = Some(frame_closure);

    // Kick off first frame
    let window = web_sys::window().expect("no window");
    {
        let slot = closure_slot.borrow();
        if let Some(cb) = slot.as_ref() {
            window.request_animation_frame(cb.as_ref().unchecked_ref()).unwrap();
        }
    }

    RafHandle { cancelled }

}

// ── Tooltip text formatting per UI-SPEC copywriting contract ─────────────────

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

/// Closures must stay alive as long as event listeners are registered.
/// We `forget` this struct so closures live for the page lifetime.
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
    selected_paper: RwSignal<Option<String>>,
    canvas_width: f64,
    canvas_height: f64,
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

            // Handle active interaction (panning / node dragging) before hover detection
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
            let (wx, wy) = s.viewport.screen_to_world(sx, sy);

            let dx = sx - s.drag_start_x;
            let dy = sy - s.drag_start_y;
            let was_click = dx * dx + dy * dy < 4.0; // < 2px = click

            let prev_interaction = s.interaction.clone();
            s.interaction = InteractionState::Idle;

            if was_click {
                if let Some(node_idx) = interaction::find_node_at(&s.graph.nodes, wx, wy) {
                    if matches!(prev_interaction, InteractionState::DraggingNode { .. }) {
                        // Click on a dragged (pinned) node — unpin it
                        s.graph.nodes[node_idx].pinned = false;
                    } else {
                        // Click on node — open paper drawer
                        let paper_id = s.graph.nodes[node_idx].id.clone();
                        s.graph.selected_node = Some(node_idx);
                        drop(s);
                        selected_paper.set(Some(paper_id));
                        return;
                    }
                } else {
                    // Click on background — deselect
                    s.graph.selected_node = None;
                    drop(s);
                    selected_paper.set(None);
                    return;
                }
            }
            // End of drag/pan — no additional action needed
        });

    canvas.add_event_listener_with_callback("mouseup", mouseup.as_ref().unchecked_ref())
        .unwrap();

    // ── dblclick — reset viewport to initial centered position ────────────────
    let state_dc = state.clone();
    let dblclick =
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |_event: web_sys::MouseEvent| {
            let mut s = state_dc.borrow_mut();
            s.viewport = Viewport::new(canvas_width, canvas_height);
        });

    canvas.add_event_listener_with_callback("dblclick", dblclick.as_ref().unchecked_ref())
        .unwrap();

    // ── wheel — zoom toward cursor ────────────────────────────────────────────
    let state_wh = state.clone();
    let canvas_wh = canvas_el.clone();
    let wheel =
        Closure::<dyn FnMut(web_sys::WheelEvent)>::new(move |event: web_sys::WheelEvent| {
            event.prevent_default();
            let rect = canvas_wh.get_bounding_client_rect();
            let cx = event.client_x() as f64 - rect.left();
            let cy = event.client_y() as f64 - rect.top();
            let delta = event.delta_y();
            let mut s = state_wh.borrow_mut();
            interaction::zoom_toward_cursor(&mut s.viewport, cx, cy, delta);
        });

    canvas.add_event_listener_with_callback("wheel", wheel.as_ref().unchecked_ref()).unwrap();

    // ── pointerleave — clear hover state and stop active interaction ──────────
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

    // Keep closures alive for the page lifetime (single-user local tool — acceptable).
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
