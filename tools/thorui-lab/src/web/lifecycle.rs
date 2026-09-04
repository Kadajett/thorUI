use super::SharedReport;
use super::helpers::{document, now, window};
use crate::report::LifecycleEvent;
use wasm_bindgen::JsValue;

const MAX_EVENTS: usize = 256;

pub fn install(report: &SharedReport) -> Result<(), JsValue> {
    for name in ["visibilitychange", "fullscreenchange"] {
        let shared = report.clone();
        let event_name = name.to_owned();
        super::helpers::listen(document()?.as_ref(), name, move |_| {
            record(&shared, &event_name);
        })?;
    }
    for name in [
        "focus",
        "blur",
        "resize",
        "orientationchange",
        "pageshow",
        "pagehide",
        "online",
        "offline",
        "beforeinstallprompt",
        "appinstalled",
    ] {
        let shared = report.clone();
        let event_name = name.to_owned();
        super::helpers::listen(window()?.as_ref(), name, move |_| {
            if event_name == "resize" || event_name == "orientationchange" {
                let _ = super::environment::observe(&shared);
            }
            record(&shared, &event_name);
        })?;
    }
    record(report, "started");
    Ok(())
}

fn record(report: &SharedReport, event: &str) {
    let visibility = document().map_or_else(
        |_| "Unknown".to_owned(),
        |doc| format!("{:?}", doc.visibility_state()),
    );
    let focused = document()
        .and_then(|doc| doc.has_focus())
        .unwrap_or_default();
    let mut target = report.borrow_mut();
    target.lifecycle.push(LifecycleEvent {
        sampled_at_ms: now(),
        event: event.to_owned(),
        visibility,
        focused,
    });
    if target.lifecycle.len() > MAX_EVENTS {
        target.lifecycle.remove(0);
    }
    drop(target);
    super::render::refresh(report);
}
