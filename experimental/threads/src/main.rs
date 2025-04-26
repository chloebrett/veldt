mod app;
mod audio_player;

use crate::app::*;
use crate::audio_player::*;
use std::time::Duration;
use wasm_bindgen::prelude::*;

use wasm_thread as thread;

use eframe::web_sys;

fn main() {
    console_log::init().unwrap();
    console_error_panic_hook::set_once();

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
