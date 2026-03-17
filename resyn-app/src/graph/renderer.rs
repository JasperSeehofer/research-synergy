use wasm_bindgen::JsCast;

use super::canvas_renderer::Canvas2DRenderer;
use super::layout_state::GraphState;
use super::webgl_renderer::WebGL2Renderer;

pub struct Viewport {
    pub offset_x: f64,
    pub offset_y: f64,
    pub scale: f64,
}

impl Viewport {
    pub fn new(canvas_width: f64, canvas_height: f64) -> Self {
        Self {
            offset_x: canvas_width / 2.0,
            offset_y: canvas_height / 2.0,
            scale: 1.0,
        }
    }

    pub fn screen_to_world(&self, sx: f64, sy: f64) -> (f64, f64) {
        ((sx - self.offset_x) / self.scale, (sy - self.offset_y) / self.scale)
    }

    pub fn world_to_screen(&self, wx: f64, wy: f64) -> (f64, f64) {
        (wx * self.scale + self.offset_x, wy * self.scale + self.offset_y)
    }

    pub fn apply(&self, ctx: &web_sys::CanvasRenderingContext2d) {
        ctx.set_transform(self.scale, 0.0, 0.0, self.scale, self.offset_x, self.offset_y)
            .unwrap();
    }
}

pub trait Renderer {
    fn draw(&mut self, state: &GraphState, viewport: &Viewport);
    fn resize(&mut self, width: u32, height: u32);
}

pub const WEBGL_THRESHOLD: usize = 300;

/// Select the best renderer for the given canvas and node count.
/// Probes WebGL2 on a temporary canvas to avoid contaminating the main canvas
/// context (once "2d" is called, WebGL2 returns null on the same canvas).
pub fn make_renderer(
    canvas: &web_sys::HtmlCanvasElement,
    node_count: usize,
) -> Box<dyn Renderer> {
    // Probe WebGL2 availability on a disposable canvas
    let document = web_sys::window().unwrap().document().unwrap();
    let probe: web_sys::HtmlCanvasElement = document
        .create_element("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();
    let can_use_webgl = probe.get_context("webgl2").ok().flatten().is_some();

    if node_count > WEBGL_THRESHOLD && can_use_webgl {
        Box::new(WebGL2Renderer::new(canvas))
    } else {
        if node_count > WEBGL_THRESHOLD {
            web_sys::console::warn_1(&"WebGL2 unavailable, falling back to Canvas 2D".into());
        }
        Box::new(Canvas2DRenderer::new(canvas))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_new_centers() {
        let vp = Viewport::new(800.0, 600.0);
        assert_eq!(vp.offset_x, 400.0);
        assert_eq!(vp.offset_y, 300.0);
        assert_eq!(vp.scale, 1.0);
    }

    #[test]
    fn test_screen_to_world_identity() {
        let vp = Viewport { offset_x: 400.0, offset_y: 300.0, scale: 1.0 };
        let (wx, wy) = vp.screen_to_world(400.0, 300.0);
        assert!((wx).abs() < 1e-10);
        assert!((wy).abs() < 1e-10);
    }

    #[test]
    fn test_screen_to_world_scale2() {
        let vp = Viewport { offset_x: 400.0, offset_y: 300.0, scale: 2.0 };
        let (wx, wy) = vp.screen_to_world(500.0, 400.0);
        assert!((wx - 50.0).abs() < 1e-10);
        assert!((wy - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_world_to_screen_round_trip() {
        let vp = Viewport { offset_x: 400.0, offset_y: 300.0, scale: 1.5 };
        let (sx, sy) = vp.world_to_screen(100.0, -50.0);
        let (wx, wy) = vp.screen_to_world(sx, sy);
        assert!((wx - 100.0).abs() < 1e-10);
        assert!((wy - (-50.0)).abs() < 1e-10);
    }
}
