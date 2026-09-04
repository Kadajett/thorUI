mod active;
mod environment;
mod frames;
mod helpers;
mod input;
mod lifecycle;
mod messaging;
mod render;

use crate::report::CapabilityReport;
use helpers::{capture_id, query_value};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

pub type SharedReport = Rc<RefCell<CapabilityReport>>;

#[wasm_bindgen(start)]
#[allow(clippy::missing_errors_doc)]
pub fn start() -> Result<(), JsValue> {
    let role = query_value("surface").unwrap_or_else(|| "main".to_owned());
    let session = query_value("session").unwrap_or_else(capture_id);
    let report = Rc::new(RefCell::new(CapabilityReport::new(
        capture_id(),
        role,
        session,
    )));
    environment::observe(&report)?;
    lifecycle::install(&report)?;
    input::install(&report)?;
    let peer = messaging::install(&report)?;
    active::install(&report)?;
    frames::install(&report)?;
    render::install_export(&report)?;
    render::refresh(&report);
    messaging::announce(&peer, &report)?;
    Ok(())
}
