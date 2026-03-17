use leptos::mount::mount_to_body;
use wasm_bindgen::prelude::wasm_bindgen;

mod app;
pub mod components;
pub mod graph;
pub mod layout;
pub mod pages;
pub mod server_fns;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(app::App);
}
