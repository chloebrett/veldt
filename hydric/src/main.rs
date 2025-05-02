mod async_state;
mod audio_state;
mod components;
mod data_state;
mod local_state;
mod playback;
mod promise;
mod rpc;
mod transform;
mod view;
mod widget;
mod window_state;

use crate::components::App;
use async_state::*;
use audio_state::*;
use data_state::*;
use eframe::web_sys;
use local_state::*;
use window_state::*;

fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("egui_canvas")
            .expect("Failed to find egui_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let _start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
            )
            .await;
    });
}
