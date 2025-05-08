use wasm_bindgen::prelude::*;
use web_sys::{ClipboardEvent, HtmlInputElement, HtmlTextAreaElement, window};

#[wasm_bindgen]
pub fn clipboard_handling() {
    let window = window().expect("No window");
    let document = window.document().expect("No document");

    let closure_copy = Closure::wrap(Box::new(move |event: ClipboardEvent| {
        let target = event.target().unwrap();

        if let Some(input) = target.dyn_ref::<HtmlInputElement>() {
            log::info!("Copy triggered on input: {}", input.value());
        } else if let Some(textarea) = target.dyn_ref::<HtmlTextAreaElement>() {
            log::info!("Copy triggered on textarea: {}", textarea.value());
        }
    }) as Box<dyn FnMut(ClipboardEvent)>);

    document
        .add_event_listener_with_callback("copy", closure_copy.as_ref().unchecked_ref())
        .expect("Failed to add copy event listener");

    closure_copy.forget();
}
