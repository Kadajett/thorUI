mod active;
mod environment;
mod frames;
mod helpers;
mod input;
mod lifecycle;
mod messaging;
mod navigation;
mod render;
mod showcase;
mod suite;
mod upload;

use crate::report::CapabilityReport;
use helpers::{capture_id, query_value};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlSelectElement;

pub type SharedReport = Rc<RefCell<CapabilityReport>>;

#[wasm_bindgen(start)]
#[allow(clippy::missing_errors_doc)]
pub fn start() -> Result<(), JsValue> {
    let role = query_value("surface").unwrap_or_else(|| "main".to_owned());
    if query_value("mode").as_deref() != Some("lab") {
        return showcase::start(role);
    }
    configure_refresh(&role)?;
    let session = query_value("session").unwrap_or_else(capture_id);
    let report = Rc::new(RefCell::new(CapabilityReport::new(
        capture_id(),
        role,
        session,
    )));
    environment::observe(&report)?;
    lifecycle::install(&report)?;
    input::install(&report)?;
    navigation::install();
    let peer = messaging::install(&report)?;
    active::install(&report)?;
    frames::install(&report)?;
    render::install_export(&report)?;
    render::install_notes(&report)?;
    suite::install(&report)?;
    render::refresh(&report);
    messaging::announce(&peer, &report)?;
    Ok(())
}

fn configure_refresh(role: &str) -> Result<(), JsValue> {
    if role == "companion" {
        let select: HtmlSelectElement = helpers::element("expected-refresh")?.dyn_into()?;
        select.set_value("60");
    }
    Ok(())
}
