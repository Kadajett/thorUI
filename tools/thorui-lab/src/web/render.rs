use super::SharedReport;
use super::helpers::{button, document, element, set_html, set_text};
use crate::report::{CapabilityReport, SupportMatrix};
use js_sys::Array;
use std::fmt::Write;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

pub fn refresh(shared: &SharedReport) {
    let report = shared.borrow();
    render_header(&report);
    render_capabilities(&report.support);
    render_controller(&report);
    render_peer(&report);
    render_lifecycle(&report);
    let json = serde_json::to_string_pretty(&*report).unwrap_or_else(|error| error.to_string());
    set_text("report-json", &json);
}

pub fn install_export(report: &SharedReport) -> Result<(), JsValue> {
    let button = button("export-report")?;
    let shared = report.clone();
    super::helpers::listen(button.as_ref(), "click", move |_| {
        if let Err(error) = export(&shared.borrow()) {
            set_text("summary-copy", &format!("Export failed: {error:?}"));
        }
    })
}

fn render_header(report: &CapabilityReport) {
    let surface = &report.surface;
    set_text("surface-role", &format!("surface: {}", surface.role));
    set_text(
        "build-revision",
        &format!("revision: {}", report.build.revision),
    );
    set_text(
        "connection-badge",
        if report.peer_link.connected {
            "Peer linked"
        } else {
            "Lab ready"
        },
    );
    if let Ok(badge) = element("connection-badge") {
        badge.set_class_name("badge good");
    }
    set_text(
        "metric-viewport",
        &format!(
            "{} × {} CSS",
            surface.viewport_width_css, surface.viewport_height_css
        ),
    );
    set_text(
        "metric-screen",
        &format!(
            "{} × {} CSS",
            surface.screen_width_css, surface.screen_height_css
        ),
    );
    set_text(
        "metric-density",
        &format!("{:.2}×", surface.device_pixel_ratio),
    );
    if let Some(run) = report.frame_runs.last() {
        set_text(
            "metric-frame",
            &format!(
                "{:.2} Hz / p95 {:.2} ms",
                run.distribution.estimated_hz, run.distribution.p95_ms
            ),
        );
    }
    set_text(
        "summary-title",
        &format!("{} surface observed", title_case(&surface.role)),
    );
    set_text(
        "summary-copy",
        &format!(
            "{}×{} viewport at {:.2}× density. {} controller sample(s), {} frame run(s).",
            surface.viewport_width_css,
            surface.viewport_height_css,
            surface.device_pixel_ratio,
            report.controllers.len(),
            report.frame_runs.len()
        ),
    );
}

fn render_capabilities(support: &SupportMatrix) {
    let entries = [
        ("BroadcastChannel", support.broadcast_channel),
        ("Gamepad", support.gamepad),
        ("Pointer Events", support.pointer_events),
        ("Fullscreen", support.fullscreen),
        ("Window Management", support.window_management),
        ("Presentation", support.presentation),
        ("Wake Lock", support.wake_lock),
        ("Service Worker", support.service_worker),
        ("Storage estimate", support.storage_manager),
        ("WebGL2", support.webgl2),
        ("WebGPU", support.webgpu),
        ("AudioContext", support.audio_context),
        ("OffscreenCanvas", support.offscreen_canvas),
        ("Installed mode", support.installed_display_mode),
    ];
    let mut html = String::new();
    for (name, value) in entries {
        let _ = write!(
            html,
            "<div><dt>{name}</dt><dd class=\"{}\">{}</dd></div>",
            yes_no_class(value),
            yes_no(value)
        );
    }
    set_html("capability-list", &html);
}

fn render_controller(report: &CapabilityReport) {
    let Some(controller) = report.controllers.last() else {
        return;
    };
    let pressed = controller
        .buttons
        .iter()
        .filter(|button| button.pressed)
        .map(|button| button.index.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    set_text(
        "controller-summary",
        &format!(
            "{}\nindex {} · mapping {} · {} buttons · {} axes\npressed: [{}]\nvibration: {}",
            controller.id,
            controller.index,
            controller.mapping,
            controller.buttons.len(),
            controller.axes.len(),
            pressed,
            yes_no(controller.vibration_supported)
        ),
    );
}

fn render_peer(report: &CapabilityReport) {
    let peer = &report.peer_link;
    if peer.peer_id.is_empty() {
        return;
    }
    set_text(
        "peer-summary",
        &format!(
            "{} ({})\nsent {} · received {} · reordered {}\nRTT median {:.2} ms · p95 {:.2} ms",
            peer.peer_id,
            peer.peer_role,
            peer.sent,
            peer.received,
            peer.lost_or_reordered,
            peer.round_trip_ms.median_ms,
            peer.round_trip_ms.p95_ms
        ),
    );
}

fn render_lifecycle(report: &CapabilityReport) {
    let mut html = String::new();
    for event in report.lifecycle.iter().rev().take(40) {
        let _ = write!(
            html,
            "<li>{:.0} ms · {} · {} · focus {}</li>",
            event.sampled_at_ms, event.event, event.visibility, event.focused
        );
    }
    set_html("lifecycle-log", &html);
}

fn export(report: &CapabilityReport) -> Result<(), JsValue> {
    let json = serde_json::to_string_pretty(report)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    let values = Array::new();
    values.push(&JsValue::from_str(&json));
    let options = BlobPropertyBag::new();
    options.set_type("application/json");
    let blob = Blob::new_with_str_sequence_and_options(&values, &options)?;
    let url = Url::create_object_url_with_blob(&blob)?;
    let anchor: HtmlAnchorElement = document()?.create_element("a")?.dyn_into()?;
    anchor.set_href(&url);
    anchor.set_download(&format!("thorui-capability-{}.json", report.capture_id));
    anchor.click();
    Url::revoke_object_url(&url)
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn yes_no_class(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}
