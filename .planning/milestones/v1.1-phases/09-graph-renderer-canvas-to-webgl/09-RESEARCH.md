# Phase 9: Graph Renderer (Canvas to WebGL) - Research

**Researched:** 2026-03-17
**Domain:** Rust/WASM canvas graphics, WebGL2 rendering, Web Worker force layout
**Confidence:** HIGH (core web-sys/wasm-bindgen patterns verified via official docs; Barnes-Hut via algorithm docs; Trunk worker support verified via issue tracker)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **No JavaScript graph libraries.** Full Rust/WASM stack only — web-sys Canvas2D/WebGL2, no sigma.js, d3, force-graph.js.
- **Renderer trait.** Shared `Renderer` trait implemented by `Canvas2DRenderer` and `WebGL2Renderer` — clean runtime switching, both maintained.
- **Automatic Canvas-to-WebGL switch** by node count: Canvas 2D under ~200–500 nodes, WebGL2 above. WebGL2 support detected on init; fall back to Canvas 2D if unavailable (log warning).
- **Barnes-Hut O(n log n) force layout** in Rust/WASM Web Worker — already project-level decision.
- **Interactions:** Click = open Phase 8 paper drawer. Hover = tooltip near cursor (title, authors, year). Drag = pin node (stops simulation for that node). Click pinned = unpin (restarts for that node). Click background + drag = pan. Scroll = zoom.
- **Edge rendering:** Arrowheads on target end. Contradiction = red solid. ABC-bridge = orange dashed. Special edges on top of regular. Default visible, togglable. Regular = thin gray low opacity.
- **Edge hover tooltip:** Regular = "A cites B". Special = gap finding summary (shared_terms, confidence, justification snippet).
- **Layout & convergence:** Animate from random start. Play/pause button. Dragging a pinned node does NOT restart simulation. Node size by citation_count. Labels = "first author + year".
- **App integration:** New "Graph" nav item in Phase 8 sidebar (after Methods). Full-page canvas with overlay controls (edge toggles, play/pause, zoom controls). Paper drawer slides in from right over canvas.

### Claude's Discretion

- Exact Barnes-Hut theta parameter and force layout tuning constants
- Web Worker message protocol and serialization format
- Canvas 2D drawing implementation details (arrowheads, node circles)
- WebGL2 shader programs and buffer management
- Exact node count threshold for Canvas-to-WebGL switch
- Zoom level at which labels appear/disappear
- CSS for graph controls overlay
- How to serialize/deserialize graph data between main thread and Web Worker

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| GRAPH-01 | Canvas 2D renderer via web-sys with Web Worker force layout (full Rust/WASM) | web-sys Canvas2D patterns verified; Trunk rust-worker confirmed; gloo-worker 0.5 available |
| GRAPH-02 | Pan/zoom/hover interactions matching current egui feature set | web-sys MouseEvent/PointerEvent patterns verified; coordinate transform math documented |
| GRAPH-03 | WebGL2 upgrade via web-sys for 1000+ node rendering (full Rust) | WebGL2 features and instanced drawing patterns documented; shader circle technique verified |
| GRAPH-04 | Barnes-Hut O(n log n) force layout in Rust/WASM replacing fdg | Algorithm fully understood; Rust implementation strategy documented; quadtree data structure specified |

</phase_requirements>

---

## Summary

Phase 9 implements a graph renderer that is entirely Rust/WASM — no JavaScript graph libraries. The project already has web-sys 0.3.91 and wasm-bindgen 0.2.114 in its transitive dependency tree via leptos-use. The core work is three things: (1) a Canvas 2D rendering pipeline, (2) a Barnes-Hut force layout running in a Web Worker, and (3) a WebGL2 renderer for the 1000-node case. These three subsystems connect via a `Renderer` trait and a structured message protocol.

The existing Leptos app shell (Phase 8) integrates cleanly: a `/graph` route with a full-page `<canvas>` element accessed via `NodeRef`, event listeners attached imperatively in an `Effect`, and the same `Drawer` + `SelectedPaper` context signal used for node click. The force layout worker is a separate Rust binary compiled with Trunk's `rust-worker` asset type (`data-type="worker"` in index.html).

The main technical challenge — not a blocker — is the animation loop pattern: `requestAnimationFrame` in Rust/WASM requires `Rc<RefCell<Option<Closure<dyn FnMut()>>>>` to allow the closure to reference itself recursively. This is a known idiom with official documentation and multiple 2024 examples. The Web Worker integration requires a separate WASM binary target; Trunk 0.21.14 (already installed) fully supports `rust-worker`.

**Primary recommendation:** Build in this order: (1) GraphData server function + data model, (2) Barnes-Hut worker binary, (3) Canvas2DRenderer, (4) interaction layer, (5) WebGL2Renderer. Each layer is independently testable and the earlier layers validate the integration before tackling WebGL2.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| web-sys | 0.3.91 | Canvas2D, WebGL2, Worker, MouseEvent, PointerEvent bindings | Already in dep tree; official WASM Web API bindings |
| wasm-bindgen | 0.2.114 | Rust↔JS interop, Closure type, RAF handle | Already in dep tree; official toolchain |
| js-sys | 0.3.91 | Float32Array for WebGL buffers, Array for setLineDash | Already in dep tree; companion to web-sys |
| gloo-worker | 0.5.0 | Web Worker lifecycle, message protocol, reactor/oneshot macros | Highest-level abstraction for WASM workers; active maintenance |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde_json | 1.0 | Serialize graph data for server fn and worker messages | Already in workspace; standard serialization |
| petgraph (serde-1 feature) | 0.7.0 | StableGraph serialization for server fn response | Already in workspace; serde-1 feature enables JSON |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| gloo-worker | raw web-sys Worker + Closure | gloo-worker handles registration, message dispatch, bincode encoding; raw approach is more control but ~3x boilerplate |
| serde_json for worker messages | bincode | bincode is gloo-worker's default (faster); JSON is easier to debug; either works — gloo-worker's codec is configurable |
| web-sys WebGL2 | wgpu (WebGPU) | wgpu is future-proof but WebGPU is less supported in Firefox 2025; WebGL2 is universally supported |

**Installation (additions to resyn-app/Cargo.toml):**
```toml
gloo-worker = { version = "0.5", features = ["futures"] }
js-sys = "0.3"

[dependencies.web-sys]
version = "0.3"
features = [
  # Canvas2D
  "CanvasRenderingContext2d",
  "HtmlCanvasElement",
  "TextMetrics",
  # WebGL2
  "WebGl2RenderingContext",
  "WebGlBuffer",
  "WebGlProgram",
  "WebGlShader",
  "WebGlUniformLocation",
  "WebGlVertexArrayObject",
  # Events
  "MouseEvent",
  "PointerEvent",
  "WheelEvent",
  "EventTarget",
  # Worker
  "Worker",
  "WorkerGlobalScope",
  "DedicatedWorkerGlobalScope",
  # DOM
  "Document",
  "Window",
  "Element",
  "DomRect",
]
```

**Version verification (confirmed against crates.io 2026-03-17):**
- web-sys: 0.3.91 (already in dep tree)
- wasm-bindgen: 0.2.114 (already in dep tree)
- gloo-worker: 0.5.0 (confirmed on crates.io)
- js-sys: 0.3.91 (already in dep tree)

---

## Architecture Patterns

### Recommended Project Structure
```
resyn-app/src/
├── pages/
│   └── graph.rs              # GraphPage Leptos component, canvas setup
├── components/
│   └── graph_controls.rs     # Overlay controls (play/pause, edge toggles, zoom)
├── graph/
│   ├── mod.rs
│   ├── renderer.rs           # Renderer trait definition
│   ├── canvas_renderer.rs    # Canvas2DRenderer impl
│   ├── webgl_renderer.rs     # WebGL2Renderer impl
│   ├── interaction.rs        # Pan/zoom/drag/hover hit-test state machine
│   ├── layout_state.rs       # Node positions, velocities, pinned flags
│   └── worker_bridge.rs      # gloo-worker bridge to force layout worker
└── server_fns/
    └── graph.rs              # get_graph_data server fn

resyn-worker/                 # NEW: separate crate for force layout worker
├── Cargo.toml
└── src/
    └── lib.rs                # Barnes-Hut worker impl, gloo-worker registration
```

### Pattern 1: NodeRef + Effect for Canvas Lifecycle

The canonical Leptos 0.8 pattern for canvas access. The Effect fires once after the component mounts, when `node_ref.get()` returns `Some`. The animation loop runs entirely outside Leptos's reactive graph.

```rust
// Source: https://leptos.dev/web_sys.html + https://github.com/leptos-rs/leptos/discussions/3490
#[component]
pub fn GraphPage() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();

    Effect::new(move |_| {
        let Some(canvas) = canvas_ref.get() else { return };
        // canvas is web_sys::HtmlCanvasElement — cast to HtmlCanvasElement
        let canvas: web_sys::HtmlCanvasElement = canvas.into();
        // start_render_loop takes ownership, returns a cleanup handle
        let handle = start_render_loop(canvas);
        on_cleanup(move || handle.cancel());
    });

    view! {
        <canvas node_ref=canvas_ref class="graph-canvas"/>
        <GraphControls/>
    }
}
```

### Pattern 2: requestAnimationFrame Recursive Loop

The `Rc<RefCell<Option<Closure<...>>>>` idiom is the standard Rust/WASM RAF pattern. The closure captures a clone of the `Rc`, allows it to schedule itself.

```rust
// Source: https://rustwasm.github.io/docs/wasm-bindgen/examples/request-animation-frame.html
// Source: https://tuttlem.github.io/2024/12/14/basic-animation-in-wasm-with-rust.html
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub struct RafHandle {
    id: i32,
    _closure: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
}

impl RafHandle {
    pub fn cancel(self) {
        web_sys::window()
            .unwrap()
            .cancel_animation_frame(self.id)
            .ok();
    }
}

fn start_render_loop(canvas: web_sys::HtmlCanvasElement, state: Rc<RefCell<GraphState>>) -> RafHandle {
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        // tick physics, draw frame
        state.borrow_mut().step();
        draw_frame(&canvas, &state.borrow());

        let window = web_sys::window().unwrap();
        let id = window
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
        // store id for potential cancellation
        let _ = id;
    }));

    let window = web_sys::window().unwrap();
    let id = window
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();

    RafHandle { id, _closure: g }
}
```

### Pattern 3: Renderer Trait

Clean abstraction enabling runtime selection between Canvas2D and WebGL2.

```rust
// Source: project pattern derived from locked Renderer trait decision
pub trait Renderer {
    fn draw(&mut self, state: &GraphState, viewport: &Viewport);
    fn resize(&mut self, width: u32, height: u32);
}

pub fn make_renderer(canvas: &web_sys::HtmlCanvasElement, node_count: usize) -> Box<dyn Renderer> {
    let can_use_webgl = canvas.get_context("webgl2").ok().flatten().is_some();
    if node_count > WEBGL_THRESHOLD && can_use_webgl {
        Box::new(WebGL2Renderer::new(canvas))
    } else {
        if node_count > WEBGL_THRESHOLD {
            web_sys::console::warn_1(&"WebGL2 unavailable, falling back to Canvas 2D".into());
        }
        Box::new(Canvas2DRenderer::new(canvas))
    }
}
```

### Pattern 4: Canvas 2D Drawing Operations

Key web-sys Canvas2D operations for graph rendering.

```rust
// Source: https://docs.rs/web-sys/latest/web_sys/struct.CanvasRenderingContext2d.html
// Draw a node circle with opacity
ctx.save();
ctx.set_global_alpha(alpha);      // f64 in 0.0..=1.0
ctx.begin_path();
ctx.arc(x, y, radius, 0.0, std::f64::consts::TAU).unwrap();
ctx.set_fill_style_str("#4a9eff");
ctx.fill();
ctx.restore();

// Draw a dashed edge (ABC-bridge = orange dashed)
ctx.save();
ctx.set_stroke_style_str("#ff9900");
ctx.set_line_width(2.0);
let dash_pattern: Vec<f64> = vec![6.0, 4.0];
ctx.set_line_dash(&js_sys::Array::from_iter(
    dash_pattern.iter().map(|&d| wasm_bindgen::JsValue::from_f64(d))
)).unwrap();
ctx.begin_path();
ctx.move_to(x1, y1);
ctx.line_to(x2, y2);
ctx.stroke();
ctx.restore();

// Draw text label
ctx.save();
ctx.set_font("11px monospace");
ctx.set_fill_style_str("#cccccc");
ctx.fill_text(&label, x, y).unwrap();
ctx.restore();
```

### Pattern 5: Arrowhead Drawing

Arrowheads require computing a small triangle at the target end of each edge, rotated to match the edge angle.

```rust
// Source: standard 2D canvas arrowhead math
fn draw_arrowhead(ctx: &CanvasRenderingContext2d, x1: f64, y1: f64, x2: f64, y2: f64) {
    let angle = (y2 - y1).atan2(x2 - x1);
    let size = 8.0_f64;
    ctx.begin_path();
    ctx.move_to(x2, y2);
    ctx.line_to(
        x2 - size * (angle - 0.4).cos(),
        y2 - size * (angle - 0.4).sin(),
    );
    ctx.line_to(
        x2 - size * (angle + 0.4).cos(),
        y2 - size * (angle + 0.4).sin(),
    );
    ctx.close_path();
    ctx.fill();
}
```

### Pattern 6: Pan/Zoom Coordinate Transform

Pan and zoom are stored as a `Viewport` struct (offset_x, offset_y, scale). All canvas coordinates are transformed before hit-testing and drawing.

```rust
// Source: https://harrisonmilbradt.com/blog/canvas-panning-and-zooming
pub struct Viewport {
    pub offset_x: f64,
    pub offset_y: f64,
    pub scale: f64,
}

impl Viewport {
    // Convert screen (event) coordinates → world coordinates
    pub fn screen_to_world(&self, sx: f64, sy: f64) -> (f64, f64) {
        ((sx - self.offset_x) / self.scale, (sy - self.offset_y) / self.scale)
    }
    // Apply to canvas context before drawing
    pub fn apply(&self, ctx: &CanvasRenderingContext2d) {
        ctx.set_transform(self.scale, 0.0, 0.0, self.scale, self.offset_x, self.offset_y).unwrap();
    }
}

// Mouse coordinate extraction (account for canvas bounding rect offset)
fn canvas_coords(event: &web_sys::MouseEvent, canvas: &web_sys::HtmlCanvasElement) -> (f64, f64) {
    let rect = canvas.get_bounding_client_rect();
    (event.client_x() as f64 - rect.left(), event.client_y() as f64 - rect.top())
}

// Zoom toward cursor (scroll event)
fn zoom_toward_cursor(vp: &mut Viewport, cx: f64, cy: f64, delta: f64) {
    let factor = if delta < 0.0 { 1.1 } else { 0.9 };
    // Adjust offset so the point under the cursor stays fixed
    vp.offset_x = cx - factor * (cx - vp.offset_x);
    vp.offset_y = cy - factor * (cy - vp.offset_y);
    vp.scale *= factor;
}
```

### Pattern 7: WebGL2 Circle Instancing

For 1000+ nodes, draw circles as instanced quads with a fragment shader that discards outside-radius pixels. This is the standard approach on webgl2fundamentals.org.

```glsl
// Vertex shader — instanced quads centered at node position
attribute vec2 a_quad;       // local quad corner: [-1,-1] to [1,1]
attribute vec2 a_position;   // per-instance: node world position
attribute float a_radius;    // per-instance: node radius
attribute float a_alpha;     // per-instance: opacity (for highlighting)

uniform mat3 u_transform;    // pan/zoom transform

varying vec2 v_local;
varying float v_alpha;

void main() {
    v_local = a_quad;
    v_alpha = a_alpha;
    vec2 world = a_position + a_quad * a_radius;
    vec3 clip = u_transform * vec3(world, 1.0);
    gl_Position = vec4(clip.xy, 0.0, 1.0);
}

// Fragment shader — discard corners to make circle
precision mediump float;
varying vec2 v_local;
varying float v_alpha;
uniform vec4 u_color;

void main() {
    float d = length(v_local);
    if (d > 1.0) discard;
    gl_FragColor = vec4(u_color.rgb, u_color.a * v_alpha);
}
```

```rust
// Source: https://webgl2fundamentals.org/webgl/lessons/webgl-instanced-drawing.html
// Set instanced data: draw_arrays_instanced
gl.vertex_attrib_divisor_with_uint_and_uint(a_position_loc, 1);  // advance once per instance
gl.vertex_attrib_divisor_with_uint_and_uint(a_radius_loc, 1);
gl.draw_arrays_instanced_with_i32_and_i32_and_i32(
    WebGl2RenderingContext::TRIANGLE_FAN,
    0,
    circle_vertex_count,
    node_count as i32,
);
```

### Pattern 8: Web Worker with gloo-worker

The force layout worker is a `#[reactor]` that receives graph topology + current positions and sends back updated positions after N simulation ticks. The worker is compiled as a separate WASM binary via Trunk's `rust-worker` asset.

```rust
// resyn-worker/src/lib.rs
use gloo_worker::reactor::{reactor, ReactorScope};
use futures::StreamExt;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LayoutInput {
    pub nodes: Vec<NodeData>,  // id, x, y, mass, pinned
    pub edges: Vec<(usize, usize)>,
    pub ticks: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LayoutOutput {
    pub positions: Vec<(f64, f64)>,   // updated x, y per node index
    pub converged: bool,
}

#[reactor]
pub async fn ForceLayoutWorker(mut scope: ReactorScope<LayoutInput, LayoutOutput>) {
    while let Some(input) = scope.next().await {
        let output = run_barnes_hut_ticks(&input);
        scope.send(output);
    }
}
```

```rust
// resyn-app/src/graph/worker_bridge.rs — main thread side
use gloo_worker::reactor::ReactorBridge;
let bridge = ForceLayoutWorker::spawner()
    .callback(move |output: LayoutOutput| {
        // update shared GraphState positions
    })
    .spawn("./force_layout_worker.js"); // path matches Trunk output
```

```html
<!-- resyn-app/index.html — tell Trunk to build the worker -->
<link data-trunk rel="rust" href="../resyn-worker/Cargo.toml" data-type="worker"/>
```

### Pattern 9: Barnes-Hut Force Layout Algorithm

The Barnes-Hut quadtree algorithm for O(n log n) repulsion. Combined with Fruchterman-Reingold attractive forces on edges and a centering force. Runs in the worker for N ticks per message, then sends positions back.

```rust
// Source: https://jheer.github.io/barnes-hut/
// Core data structures
struct QuadTree {
    bounds: Rect,
    center_of_mass: (f64, f64),
    total_mass: f64,
    children: Option<Box<[QuadTree; 4]>>,
    node_idx: Option<usize>,  // leaf only
}

// Theta = 0.9 (d3-force default); range 0.5..=1.5
const THETA: f64 = 0.9;

fn barnes_hut_repulsion(tree: &QuadTree, node: (f64, f64), mass: f64) -> (f64, f64) {
    let dx = tree.center_of_mass.0 - node.0;
    let dy = tree.center_of_mass.1 - node.1;
    let dist = (dx*dx + dy*dy).sqrt().max(1.0);

    if tree.children.is_none() || tree.bounds.width / dist < THETA {
        // Use this node/cell as a single force source
        let force = REPULSION_CONSTANT * mass * tree.total_mass / (dist * dist);
        (-dx / dist * force, -dy / dist * force)
    } else {
        // Recurse into children
        tree.children.as_ref().unwrap().iter()
            .map(|child| barnes_hut_repulsion(child, node, mass))
            .fold((0.0, 0.0), |acc, f| (acc.0 + f.0, acc.1 + f.1))
    }
}
```

**Recommended force constants (based on d3-force defaults, tunable via Claude's discretion):**
- Repulsion strength: `-30.0` (negative = repulsive)
- Attraction strength on edges: `0.1` (spring-like)
- Center gravity: `0.05`
- Velocity damping (alpha decay): multiply by `0.92` per tick
- Convergence threshold: alpha < `0.001`

### Pattern 10: GraphData Server Function

The server function returns a flat data transfer struct (not raw StableGraph) to minimize serialization overhead and keep the client-side data model clean.

```rust
// resyn-app/src/server_fns/graph.rs
#[derive(Serialize, Deserialize, Clone)]
pub struct GraphNode {
    pub id: String,             // arXiv ID
    pub title: String,
    pub authors: Vec<String>,
    pub year: String,           // first 4 chars of published
    pub citation_count: Option<u32>,
    pub abstract_text: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GraphEdge {
    pub from: String,           // arXiv ID
    pub to: String,
    pub edge_type: EdgeType,    // Regular, Contradiction, AbcBridge
    // For special edges only:
    pub shared_terms: Vec<String>,
    pub confidence: Option<f32>,
    pub justification: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum EdgeType { Regular, Contradiction, AbcBridge }

#[derive(Serialize, Deserialize, Clone)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[server(GetGraphData, "/api")]
pub async fn get_graph_data() -> Result<GraphData, ServerFnError> { ... }
```

### Anti-Patterns to Avoid
- **Sending petgraph StableGraph directly over server fn:** It serializes but includes `node_holes` and internal indices that don't map cleanly to rendering. Use a flat DTO struct.
- **Mixing Leptos reactive signals with RAF:** Leptos signal reads inside RAF closures cause unexpected reactive subscriptions. Keep the render loop state outside Leptos's reactive graph in `Rc<RefCell<GraphState>>`.
- **Calling canvas `clear_rect` without resetting transforms:** Always `ctx.set_transform(1, 0, 0, 1, 0, 0)` before `clearRect(0, 0, w, h)`, then reapply the viewport transform.
- **Using `Closure::once` for RAF:** RAF callbacks fire repeatedly; must use `Closure::new` (perpetual) not `Closure::once`.
- **Forgetting to call `ctx.restore()` after `save()`:** State leaks between draw calls. Always pair save/restore around style changes.
- **Worker as a Leptos route/component:** The worker binary is a separate entry point that calls `ForceLayoutWorker::registrar().register()`, not a Leptos app. It MUST NOT have a Leptos app root.
- **Sharing canvas context 2D and WebGL2 on the same canvas:** Once `get_context("2d")` is called, WebGL2 context will return null and vice versa. If switching renderers, recreate the canvas element.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Web Worker lifecycle | Custom Worker::new + JS glue + message dispatch | `gloo-worker` 0.5 `#[reactor]` | Handles registration, codec, WorkerScope, bridge lifecycle |
| Worker serialization | Manual JSON encode/decode of positions | gloo-worker's bincode codec (default) | Zero-allocation, significantly faster than JSON for numeric arrays |
| requestAnimationFrame cancel | Manual handle tracking | `window.cancel_animation_frame(id)` via web-sys Window | Already supported; wrap in a `RafHandle` struct with Drop |
| Circle rendering in WebGL2 | Tessellated circle geometry (many triangles) | Instanced quad + fragment shader discard | 4 vertices per node vs. 32+; identical visual quality at all zoom levels |
| Graph layout | Custom spring algorithm from scratch | Barnes-Hut quadtree (GRAPH-04 requirement) | O(n²) spring is unusable at 500+ nodes; quadtree is the standard solution |

**Key insight:** gloo-worker is the right abstraction level here — it eliminates the JS glue file problem for Trunk builds and provides typed message dispatch. The alternative (raw `web_sys::Worker`) requires writing and shipping a separate `.js` bootstrap file.

---

## Common Pitfalls

### Pitfall 1: Trunk rust-worker Build Setup

**What goes wrong:** Trunk builds the main WASM with `--target web` (ES modules). A worker binary built the same way will fail to load in a Web Worker context because browsers do not universally support ES module workers in 2025.

**Why it happens:** Trunk's `rust-worker` asset type defaults `data-bindgen-target` to `no-modules`, which is correct for workers. But forgetting the `data-type="worker"` attribute on the `<link>` tag makes Trunk treat it as a second main binary, producing the wrong target.

**How to avoid:**
```html
<!-- CORRECT -->
<link data-trunk rel="rust" href="../resyn-worker/Cargo.toml" data-type="worker"/>
<!-- WRONG — missing data-type -->
<link data-trunk rel="rust" href="../resyn-worker/Cargo.toml"/>
```

**Warning signs:** Worker fails to initialize; console error about `import` not defined in worker scope.

### Pitfall 2: Canvas Context Contamination

**What goes wrong:** After switching from Canvas2D to WebGL2 renderer (or vice versa), the new context returns `null` from `get_context()`.

**Why it happens:** Browsers lock the context type on a `<canvas>` element. Once `get_context("2d")` is called, that canvas cannot ever give you a `webgl2` context and vice versa.

**How to avoid:** The `make_renderer()` function must probe WebGL2 availability with a *temporary* canvas or at component mount before any context is acquired. The threshold check runs once at startup. If renderer switch is needed at runtime, replace the canvas element.

**Warning signs:** `get_context("webgl2")` returns `Ok(None)` after Canvas2D was already acquired.

### Pitfall 3: RAF Closure Memory Leak

**What goes wrong:** The RAF loop continues after the component unmounts, causing a panic (accessing freed DOM) or silent infinite loop.

**Why it happens:** The `Rc<RefCell<Option<Closure>>>` is self-referential. If not explicitly cancelled, it holds itself alive indefinitely.

**How to avoid:** Return a `RafHandle` from `start_render_loop`. In the Effect, call `on_cleanup(move || handle.cancel())`. `cancel_animation_frame` cancels the pending frame and the closure is dropped when the `RafHandle` drops.

**Warning signs:** Leptos component remounts cause errors about dead DOM nodes; browser memory grows unboundedly.

### Pitfall 4: Force Layout Worker Blocking Main Thread

**What goes wrong:** Barnes-Hut with 1000 nodes at 60 fps cannot run synchronously on the main thread — it will drop frames and block UI events.

**Why it happens:** Even at O(n log n), 1000 nodes × 60 fps = significant CPU budget. The main thread is single-threaded in WASM.

**How to avoid:** The worker receives the full node/edge data once, then runs N ticks internally per message. Recommended: send a `LayoutInput` per animation frame with `ticks: 1`, or batch multiple ticks (e.g., `ticks: 3`) and accept slightly coarser animation. The worker sends back `LayoutOutput` which the main thread applies before the next draw.

**Warning signs:** Frame rate drops to <10 fps with >200 nodes; `performance.now()` shows >16ms in the main thread per frame.

### Pitfall 5: Coordinate System Mismatch Between Worker and Renderer

**What goes wrong:** Node positions computed in the worker's "world space" don't match the viewport transform applied by the renderer. Nodes appear off-screen or clustered at origin.

**Why it happens:** The worker operates in an abstract simulation space (e.g., -500 to +500 units). The renderer applies a viewport transform. If the initial positions or simulation scale don't match the canvas size, the graph appears tiny or off-canvas.

**How to avoid:** Initialize node positions in world space centered around (0, 0) with spread proportional to `sqrt(node_count) * 50` pixels. The viewport's initial scale and offset should center (0, 0) to the canvas center. Send canvas dimensions to the worker in `LayoutInput` so the simulation uses matching world-space units.

**Warning signs:** On first render, all nodes appear in one corner or the canvas is blank.

### Pitfall 6: WebGL2 State Machine Leaks

**What goes wrong:** WebGL2 draw calls produce incorrect output after the second frame because vertex buffer bindings or shader programs are left bound from a previous draw.

**Why it happens:** WebGL2 is a stateful API. Every `bind_buffer`, `use_program`, and `bind_vertex_array` call changes global GL state. Forgetting to unbind after a draw pass contaminates subsequent draw calls.

**How to avoid:** Structure draws as: bind VAO → set uniforms → draw → unbind VAO. Use `bind_vertex_array(None)` and `use_program(None)` at the end of each draw pass. Prefer Vertex Array Objects (VAOs) — WebGL2 requires them for attribute configuration anyway.

**Warning signs:** Nodes or edges render correctly on frame 1 but corrupted on frame 2+.

---

## Code Examples

### GraphPage Component Skeleton

```rust
// Source: Leptos NodeRef + Effect pattern (book.leptos.dev/web_sys.html)
#[component]
pub fn GraphPage() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    let graph_data = Resource::new(|| (), |_| get_graph_data());

    Effect::new(move |_| {
        let Some(canvas) = canvas_ref.get() else { return };
        let canvas: web_sys::HtmlCanvasElement = canvas.into();

        // Resize canvas to fill container
        let width = canvas.offset_width() as u32;
        let height = canvas.offset_height() as u32;
        canvas.set_width(width);
        canvas.set_height(height);

        // Wait for graph data
        let Some(Ok(data)) = graph_data.get() else { return };

        let handle = start_graph(canvas, data);
        on_cleanup(move || drop(handle));
    });

    view! {
        <div class="graph-page">
            <canvas node_ref=canvas_ref class="graph-canvas" style="width:100%;height:100%;"/>
            <GraphControls/>
        </div>
    }
}
```

### Hit-Test for Node Hover

```rust
// Source: standard 2D circle hit-test
fn find_node_at(
    nodes: &[NodeState],
    screen_x: f64,
    screen_y: f64,
    viewport: &Viewport,
) -> Option<usize> {
    let (wx, wy) = viewport.screen_to_world(screen_x, screen_y);
    nodes.iter().position(|n| {
        let dx = n.x - wx;
        let dy = n.y - wy;
        (dx * dx + dy * dy).sqrt() <= n.radius
    })
}
```

### WebGL2 Renderer Initialization

```rust
// Source: https://rustwasm.github.io/docs/wasm-bindgen/examples/webgl.html
fn init_webgl2(canvas: &web_sys::HtmlCanvasElement) -> Option<WebGl2RenderingContext> {
    canvas
        .get_context("webgl2")
        .ok()?
        .and_then(|obj| obj)?
        .dyn_into::<WebGl2RenderingContext>()
        .ok()
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| fdg crate (removed in Phase 6) | Custom Barnes-Hut in Rust/WASM worker | Phase 6 (WEB-05) | fdg was egui-coupled; custom impl gives full control over theta, forces, convergence |
| egui force graph visualization | web-sys Canvas2D / WebGL2 | Phase 9 | Proper browser rendering with native event handling |
| Trunk single WASM binary | Trunk rust-worker for worker binary | Trunk 0.17+ | `data-type="worker"` asset link in index.html, no separate build script needed |
| Shared memory worker (SharedArrayBuffer) | PostMessage serialization (gloo-worker bincode) | — | SharedArrayBuffer requires COOP/COEP headers; bincode is simpler and sufficient for position arrays |

**Deprecated/outdated:**
- `fdg` crate: removed in Phase 6 (WEB-05). No longer a dependency. Do not reintroduce.
- `eframe`/`egui`: removed in Phase 6 (WEB-05). All visualization is now browser-native.
- `Closure::once` for RAF: wrong type — use `Closure::new` for recurring callbacks.

---

## Open Questions

1. **Worker spawn path in Trunk output**
   - What we know: Trunk places worker WASM at a content-hashed URL like `force_layout_worker-abc123.js` in the dist directory. The gloo-worker spawn path must match this.
   - What's unclear: Trunk may support a `data-name` attribute to set a stable output filename, or there may be a build-time constant injection pattern.
   - Recommendation: In Wave 1, test with a fixed `data-bin="force_layout_worker"` and verify Trunk outputs a predictable JS filename. If Trunk hashes it, explore `data-name` attribute or check trunkrs.dev docs for stable naming.

2. **gloo-worker Trunk integration verification**
   - What we know: gloo-worker 0.5 supports reactor pattern; Trunk 0.21 supports `rust-worker` asset. The combination should work.
   - What's unclear: Whether gloo-worker's default worker bootstrap expects ES modules or no-modules target. Trunk defaults to `no-modules` for workers which should be correct.
   - Recommendation: Build a minimal spike (single-task worker that squares a number) in Wave 0 before implementing Barnes-Hut. Validates the entire build pipeline with minimal debugging surface.

3. **WebGL2 context probe strategy**
   - What we know: You cannot acquire both a 2D and WebGL2 context on the same canvas element.
   - What's unclear: Whether probing on a temporary `document.createElement("canvas")` (never attached to DOM) is reliable across all browsers.
   - Recommendation: Probe on a temporary canvas during `GraphPage` mount, before acquiring any context on the main canvas. Store result in a `use_memo` or component-level `bool`.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust unit tests, no browser runner) |
| Config file | none — cargo test standard |
| Quick run command | `cargo test graph` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GRAPH-01 | Canvas2D renderer draws without panic; RAF loop starts/stops | unit | `cargo test graph::canvas_renderer` | Wave 0 |
| GRAPH-01 | Worker receives LayoutInput, returns LayoutOutput (basic positions) | unit | `cargo test graph::layout_state` | Wave 0 |
| GRAPH-02 | screen_to_world / world_to_screen round-trips at various zoom/pan | unit | `cargo test graph::interaction::viewport` | Wave 0 |
| GRAPH-02 | find_node_at returns correct node index for given screen coordinates | unit | `cargo test graph::interaction::hit_test` | Wave 0 |
| GRAPH-03 | WebGL2 shader compiles without errors (headless not possible — skip) | manual | browser console inspection | N/A — manual only |
| GRAPH-04 | Barnes-Hut single step produces non-zero forces for two separated nodes | unit | `cargo test resyn_worker::barnes_hut` | Wave 0 |
| GRAPH-04 | Barnes-Hut 1000-node layout step completes in < 16ms (benchmarks) | bench | `cargo bench barnes_hut` | Wave 0 (optional) |
| GRAPH-04 | Force layout converges (alpha < 0.001) within 500 ticks for 100-node graph | unit | `cargo test resyn_worker::convergence` | Wave 0 |
| GRAPH-01/02/03 | Graph page renders, nodes visible, node click opens drawer | manual / browser | trunk serve + browser inspection | N/A — manual |

**Note on browser tests:** Canvas2D drawing, WebGL2 rendering, and full interaction are inherently browser-only. Unit tests cover the pure logic (coordinate transforms, hit-test, Barnes-Hut physics, force math). Browser integration is verified manually at the phase gate.

### Sampling Rate
- **Per task commit:** `cargo test graph` (unit tests for the module changed)
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green + browser smoke test (graph page renders, node click opens drawer, force layout runs)

### Wave 0 Gaps
- [ ] `resyn-worker/` crate — create with gloo-worker dependency, basic worker registration
- [ ] `resyn-app/src/graph/mod.rs` — module scaffold
- [ ] `resyn-app/src/graph/layout_state.rs` — NodeState, EdgeData, GraphState structs
- [ ] `resyn-app/src/graph/interaction.rs` — Viewport, hit_test
- [ ] Worker spike test: compile and load minimal gloo-worker via Trunk to validate build pipeline
- [ ] `resyn-app/src/server_fns/graph.rs` — GraphData, GraphNode, GraphEdge types

*(Existing infrastructure: cargo test, no new framework install needed)*

---

## Sources

### Primary (HIGH confidence)
- [wasm-bindgen Guide: web-sys canvas hello world](https://rustwasm.github.io/docs/wasm-bindgen/examples/2d-canvas.html) — Canvas2D feature list, context pattern
- [wasm-bindgen Guide: requestAnimationFrame](https://rustwasm.github.io/docs/wasm-bindgen/examples/request-animation-frame.html) — RAF recursive closure canonical pattern
- [wasm-bindgen Guide: WASM in Web Worker](https://rustwasm.github.io/docs/wasm-bindgen/examples/wasm-in-web-worker.html) — Worker::new, message passing pattern
- [wasm-bindgen Guide: WebGL](https://rustwasm.github.io/docs/wasm-bindgen/examples/webgl.html) — WebGL features, shader compile pattern
- [web_sys CanvasRenderingContext2d docs.rs](https://docs.rs/web-sys/latest/web_sys/struct.CanvasRenderingContext2d.html) — set_line_dash signature, fill_text, global_alpha
- [gloo-worker docs.rs](https://docs.rs/gloo-worker/latest/gloo_worker/) — reactor/oneshot patterns, WorkerBridge, bincode default
- [Barnes-Hut Approximation (jheer.github.io)](https://jheer.github.io/barnes-hut/) — theta parameter, quadtree structure, algorithm steps
- [WebGL2 Instanced Drawing (webgl2fundamentals.org)](https://webgl2fundamentals.org/webgl/lessons/webgl-instanced-drawing.html) — drawArraysInstanced, vertexAttribDivisor
- [Trunk issue #46](https://github.com/trunk-rs/trunk/issues/46) — Trunk rust-worker support confirmed completed/merged
- [trunkrs.dev assets](https://trunkrs.dev/guide/assets/) — `data-type="worker"`, `data-bindgen-target` attributes

### Secondary (MEDIUM confidence)
- [Leptos canvas discussion #2245](https://github.com/leptos-rs/leptos/discussions/2245) — NodeRef + Effect pattern for canvas, "use web-sys directly" recommendation
- [Leptos canvas discussion #3490](https://github.com/leptos-rs/leptos/discussions/3490) — Leptos 0.7.x NodeRef declarative pattern with Effect
- [Canvas panning and zooming (harrisonmilbradt.com)](https://harrisonmilbradt.com/blog/canvas-panning-and-zooming) — DOMMatrix getTransform, zoom toward cursor math
- [WebGL2 fastest circles (webgl2fundamentals.org)](https://webgl2fundamentals.org/webgl/lessons/webgl-qna-the-fastest-way-to-draw-many-circles.html) — instanced quad + discard pattern
- [Cogs and Levers 2024 RAF tutorial](https://tuttlem.github.io/2024/12/14/basic-animation-in-wasm-with-rust.html) — Rc<RefCell<>> pattern for RAF confirmed working pattern

### Tertiary (LOW confidence — flag for validation)
- gloo-worker + Trunk worker exact spawn path / filename stability: confirmed gloo-worker 0.5 exists and reactor pattern documented, but exact Trunk output filename for worker binary should be validated via a spike. LOW confidence on the exact `spawn("./...")` path.
- WebGL2 context probe on temporary canvas: standard practice across the web ecosystem but not found in an official wasm-bindgen/web-sys document specifically. MEDIUM-LOW; validate experimentally.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — web-sys/wasm-bindgen version confirmed in existing dep tree; gloo-worker confirmed on crates.io
- Architecture: HIGH — NodeRef+Effect pattern verified via Leptos official docs; RAF pattern from official wasm-bindgen guide; Barnes-Hut well-documented
- Pitfalls: HIGH — canvas context contamination and RAF leak are verified known issues from official sources and community; GL state machine is standard WebGL knowledge
- Validation architecture: HIGH — all test types are straightforward cargo test; browser-only concerns correctly identified as manual

**Research date:** 2026-03-17
**Valid until:** 2026-06-17 (stable ecosystem — web-sys, wasm-bindgen, gloo-worker are slow-moving)
