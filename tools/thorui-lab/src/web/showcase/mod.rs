mod drawing;
mod input;
mod sync;

use super::frames::next_frame;
use super::helpers::{document, element, query_value, set_text};
use crate::showcase::{ShowcaseAction, ShowcaseModel};
use drawing::Stage;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;

pub(super) struct Runtime {
    model: ShowcaseModel,
    stage: Stage,
    link: sync::ShowcaseLink,
    controller: input::ControllerState,
    role: String,
}

impl Runtime {
    fn new(role: String) -> Result<Self, JsValue> {
        let canvas: HtmlCanvasElement = element("lumen-canvas")?.dyn_into()?;
        let session = query_value("session").unwrap_or_else(|| "public-showcase".to_owned());
        Ok(Self {
            model: ShowcaseModel::default(),
            stage: Stage::new(canvas)?,
            link: sync::ShowcaseLink::new(&session, &role)?,
            controller: input::ControllerState::default(),
            role,
        })
    }

    pub fn apply_local(&mut self, action: ShowcaseAction) {
        self.link.send_action(&action);
        self.apply(action);
    }

    pub fn apply_remote(&mut self, action: ShowcaseAction) {
        self.apply(action);
    }

    fn apply(&mut self, action: ShowcaseAction) {
        self.model = std::mem::take(&mut self.model).apply(action);
    }

    fn frame(&mut self, timestamp: f64) {
        input::poll_controller(self);
        self.stage.draw(&self.model, &self.role, timestamp);
        render_status(&self.model);
    }
}

pub fn start(role: String) -> Result<(), JsValue> {
    configure_document(&role)?;
    let runtime = Rc::new(RefCell::new(Runtime::new(role)?));
    input::install(&runtime)?;
    sync::install(&runtime);
    runtime.borrow().link.announce();
    spawn_local(render_loop(runtime));
    Ok(())
}

async fn render_loop(runtime: Rc<RefCell<Runtime>>) {
    while let Ok(timestamp) = next_frame().await {
        runtime.borrow_mut().frame(timestamp);
    }
}

fn configure_document(role: &str) -> Result<(), JsValue> {
    let body = document()?
        .body()
        .ok_or_else(|| JsValue::from_str("document body is unavailable"))?;
    let host = query_value("host").unwrap_or_else(|| "browser".to_owned());
    body.set_class_name(&format!("showcase-mode surface-{role} host-{host}"));
    set_text("demo-surface-role", role);
    Ok(())
}

fn render_status(model: &ShowcaseModel) {
    let cursor = model.cursor();
    set_text("stroke-count", &model.marks().len().to_string());
    set_text(
        "cursor-position",
        &format!("{:02.0} · {:02.0}", cursor.x * 100.0, cursor.y * 100.0),
    );
    set_text(
        "demo-peer-status",
        if model.peer_connected() {
            "Dual surface linked"
        } else {
            "Solo surface"
        },
    );
    if let Ok(indicator) = element("active-lumen") {
        let _ = indicator.set_attribute("style", &format!("--lumen:{}", model.color().css()));
    }
}
