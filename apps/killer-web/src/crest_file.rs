use killer_assets::crest::CrestKind;
use std::sync::{Arc, Mutex};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Event, HtmlInputElement};

pub type Inbox = Arc<Mutex<Vec<(CrestKind, Option<Vec<u8>>)>>>;

pub fn pick(kind: CrestKind, inbox: &Inbox) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else { return };
    let Ok(element) = document.create_element("input") else { return };
    let Ok(input) = element.dyn_into::<HtmlInputElement>() else { return };
    input.set_type("file");
    input.set_accept(".bmp,image/bmp");
    let chosen = input.clone();
    let inbox = Arc::clone(inbox);
    let on_change = Closure::once_into_js(move |_event: Event| {
        let file = chosen.files().and_then(|files| files.item(0));
        let inbox = Arc::clone(&inbox);
        wasm_bindgen_futures::spawn_local(async move {
            let bytes = match file {
                Some(file) => match JsFuture::from(file.array_buffer()).await {
                    Ok(buffer) => Some(js_sys::Uint8Array::new(&buffer).to_vec()),
                    Err(_) => None,
                },
                None => None,
            };
            if let Ok(mut queue) = inbox.lock() {
                queue.push((kind, bytes));
            }
        });
    });
    input.set_onchange(Some(on_change.unchecked_ref()));
    input.click();
}
