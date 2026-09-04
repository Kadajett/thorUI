use js_sys::{Date, Function, Math, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Element, Event, HtmlButtonElement, Performance, Window};

pub fn window() -> Result<Window, JsValue> {
    web_sys::window().ok_or_else(|| JsValue::from_str("window is unavailable"))
}

pub fn document() -> Result<Document, JsValue> {
    window()?
        .document()
        .ok_or_else(|| JsValue::from_str("document is unavailable"))
}

pub fn element(id: &str) -> Result<Element, JsValue> {
    document()?
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("missing element: {id}")))
}

pub fn button(id: &str) -> Result<HtmlButtonElement, JsValue> {
    element(id)?.dyn_into().map_err(Into::into)
}

pub fn set_text(id: &str, value: &str) {
    if let Ok(element) = element(id) {
        element.set_text_content(Some(value));
    }
}

pub fn set_html(id: &str, value: &str) {
    if let Ok(element) = element(id) {
        element.set_inner_html(value);
    }
}

pub fn now() -> f64 {
    performance().map_or_else(|_| Date::now(), |clock| clock.now())
}

pub fn iso_now() -> String {
    Date::new_0().to_iso_string().into()
}

pub fn performance() -> Result<Performance, JsValue> {
    window()?
        .performance()
        .ok_or_else(|| JsValue::from_str("performance clock is unavailable"))
}

pub fn capture_id() -> String {
    format!("{:.0}-{:.16}", Date::now(), Math::random()).replace('.', "")
}

pub fn query_value(key: &str) -> Option<String> {
    let search = window().ok()?.location().search().ok()?;
    search.trim_start_matches('?').split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        (name == key).then(|| value.to_owned())
    })
}

pub fn property(object: &JsValue, name: &str) -> Option<JsValue> {
    Reflect::get(object, &JsValue::from_str(name)).ok()
}

pub fn string_property(object: &JsValue, name: &str) -> String {
    property(object, name)
        .and_then(|value| value.as_string())
        .unwrap_or_default()
}

pub fn number_property(object: &JsValue, name: &str) -> f64 {
    property(object, name)
        .and_then(|value| value.as_f64())
        .unwrap_or_default()
}

pub fn bool_property(object: &JsValue, name: &str) -> bool {
    property(object, name)
        .and_then(|value| value.as_bool())
        .unwrap_or_default()
}

pub fn supports(object: &JsValue, name: &str) -> bool {
    property(object, name).is_some_and(|value| !value.is_undefined() && !value.is_null())
}

pub fn call0(object: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    let function: Function = property(object, name)
        .ok_or_else(|| JsValue::from_str(&format!("{name} is unavailable")))?
        .dyn_into()?;
    function.call0(object)
}

pub fn call1(object: &JsValue, name: &str, argument: &JsValue) -> Result<JsValue, JsValue> {
    let function: Function = property(object, name)
        .ok_or_else(|| JsValue::from_str(&format!("{name} is unavailable")))?
        .dyn_into()?;
    function.call1(object, argument)
}

pub fn display_mode() -> String {
    let modes = ["fullscreen", "standalone", "minimal-ui", "browser"];
    modes
        .iter()
        .find(|mode| {
            window()
                .ok()
                .and_then(|host| {
                    host.match_media(&format!("(display-mode: {mode})"))
                        .ok()
                        .flatten()
                })
                .is_some_and(|query| query.matches())
        })
        .unwrap_or(&"unknown")
        .to_string()
}

pub fn listen<F>(target: &web_sys::EventTarget, name: &str, handler: F) -> Result<(), JsValue>
where
    F: FnMut(Event) + 'static,
{
    let closure = wasm_bindgen::closure::Closure::<dyn FnMut(Event)>::new(handler);
    target.add_event_listener_with_callback(name, closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

pub fn error_text(error: &JsValue) -> String {
    error
        .as_string()
        .or_else(|| property(error, "message").and_then(|value| value.as_string()))
        .unwrap_or_else(|| "browser rejected the request".to_owned())
}
