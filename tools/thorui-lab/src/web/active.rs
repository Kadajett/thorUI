use super::SharedReport;
use super::helpers::{
    button, call0, call1, error_text, number_property, property, set_text, string_property, window,
};
use js_sys::{Array, Promise};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{JsFuture, spawn_local};

pub fn install(report: &SharedReport) -> Result<(), JsValue> {
    install_active_probes(report)?;
    install_companion(report)?;
    install_fullscreen(report)?;
    install_wake_lock(report)?;
    register_service_worker(report);
    Ok(())
}

fn install_active_probes(report: &SharedReport) -> Result<(), JsValue> {
    let trigger = button("run-active")?;
    let control = trigger.clone();
    let shared = report.clone();
    super::helpers::listen(trigger.as_ref(), "click", move |_| {
        control.set_disabled(true);
        set_text(
            "summary-copy",
            "Running browser permission and adapter probes…",
        );
        let control = control.clone();
        let shared = shared.clone();
        spawn_local(async move {
            run(&shared).await;
            control.set_disabled(false);
        });
    })
}

pub async fn run(report: &SharedReport) {
    probe_displays(report).await;
    probe_storage(report).await;
    probe_webgpu(report).await;
    probe_audio(report).await;
    super::render::refresh(report);
}

async fn probe_displays(report: &SharedReport) {
    let Ok(host) = window() else {
        return;
    };
    let host_value: JsValue = host.into();
    match await_call0(&host_value, "getScreenDetails").await {
        Ok(details) => record_displays(report, &details),
        Err(error) => {
            report.borrow_mut().active_probes.display_details =
                format!("unavailable: {}", error_text(&error));
        }
    }
}

fn record_displays(report: &SharedReport, details: &JsValue) {
    let screens = property(details, "screens").map_or_else(Array::new, |value| Array::from(&value));
    let labels = screens
        .iter()
        .map(|screen| {
            format!(
                "{}: {}×{} at {},{} · {:.2}×",
                string_property(&screen, "label"),
                number_property(&screen, "width"),
                number_property(&screen, "height"),
                number_property(&screen, "left"),
                number_property(&screen, "top"),
                number_property(&screen, "devicePixelRatio")
            )
        })
        .collect::<Vec<_>>();
    let mut target = report.borrow_mut();
    target.active_probes.display_count = Some(screens.length() as usize);
    target.active_probes.display_details = labels.join(" | ");
}

async fn probe_storage(report: &SharedReport) {
    let Ok(host) = window() else {
        return;
    };
    let navigator: JsValue = host.navigator().into();
    let storage = property(&navigator, "storage").unwrap_or(JsValue::UNDEFINED);
    if let Ok(estimate) = await_call0(&storage, "estimate").await {
        let mut target = report.borrow_mut();
        target.active_probes.storage_usage_bytes =
            property(&estimate, "usage").and_then(|value| value.as_f64());
        target.active_probes.storage_quota_bytes =
            property(&estimate, "quota").and_then(|value| value.as_f64());
    }
}

async fn probe_webgpu(report: &SharedReport) {
    let adapter_result = async {
        let host = window()?;
        let navigator: JsValue = host.navigator().into();
        let gpu =
            property(&navigator, "gpu").ok_or_else(|| JsValue::from_str("WebGPU unavailable"))?;
        let adapter = await_call0(&gpu, "requestAdapter").await?;
        if adapter.is_null() {
            return Err(JsValue::from_str("no adapter returned"));
        }
        Ok(adapter_name(&adapter))
    }
    .await;
    report.borrow_mut().active_probes.webgpu_adapter =
        adapter_result.unwrap_or_else(|error| error_text(&error));
}

fn adapter_name(adapter: &JsValue) -> String {
    let info = property(adapter, "info").unwrap_or(JsValue::UNDEFINED);
    let parts = ["vendor", "architecture", "device", "description"]
        .iter()
        .map(|name| string_property(&info, name))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        "adapter granted".to_owned()
    } else {
        parts.join(" · ")
    }
}

async fn probe_audio(report: &SharedReport) {
    let result = (|| {
        let root = js_sys::global();
        let constructor = property(&root, "AudioContext")
            .or_else(|| property(&root, "webkitAudioContext"))
            .ok_or_else(|| JsValue::from_str("AudioContext unavailable"))?;
        let context = js_sys::Reflect::construct(
            &constructor.dyn_into::<js_sys::Function>()?,
            &Array::new(),
        )?;
        Ok(context)
    })();
    let message = match result {
        Ok(context) => match await_call0(&context, "resume").await {
            Ok(_) => {
                let state = string_property(&context, "state");
                let _ = await_call0(&context, "close").await;
                format!("resumed: {state}")
            }
            Err(error) => error_text(&error),
        },
        Err(error) => error_text(&error),
    };
    report.borrow_mut().active_probes.audio_result = message;
}

fn install_companion(report: &SharedReport) -> Result<(), JsValue> {
    let trigger = button("open-companion")?;
    let shared = report.clone();
    super::helpers::listen(trigger.as_ref(), "click", move |_| {
        let target_role = if shared.borrow().surface.role == "main" {
            "companion"
        } else {
            "main"
        };
        let session = shared.borrow().surface.session_id.clone();
        let url = format!("/?surface={target_role}&session={session}");
        let result = window().and_then(|host| host.open_with_url_and_target(&url, "_blank"));
        let note = match result {
            Ok(Some(_)) => format!("Opened {target_role} surface"),
            Ok(None) => "Popup blocked".to_owned(),
            Err(error) => format!("Popup failed: {}", error_text(&error)),
        };
        shared.borrow_mut().notes.push(note);
        super::render::refresh(&shared);
    })
}

fn install_fullscreen(report: &SharedReport) -> Result<(), JsValue> {
    let trigger = button("request-fullscreen")?;
    let shared = report.clone();
    super::helpers::listen(trigger.as_ref(), "click", move |_| {
        let shared = shared.clone();
        spawn_local(async move {
            let result = request_fullscreen().await;
            shared.borrow_mut().active_probes.fullscreen_result = result;
            super::render::refresh(&shared);
        });
    })
}

async fn request_fullscreen() -> String {
    let result = async {
        let document: JsValue = super::helpers::document()?.into();
        let active = property(&document, "fullscreenElement").is_some_and(|value| !value.is_null());
        if active {
            await_call0(&document, "exitFullscreen").await?;
        } else {
            let root = property(&document, "documentElement")
                .ok_or_else(|| JsValue::from_str("document root missing"))?;
            await_call0(&root, "requestFullscreen").await?;
        }
        Ok::<_, JsValue>(if active { "exited" } else { "entered" })
    }
    .await;
    result.map_or_else(|error| error_text(&error), str::to_owned)
}

fn install_wake_lock(report: &SharedReport) -> Result<(), JsValue> {
    let trigger = button("toggle-wake-lock")?;
    let shared = report.clone();
    let held = Rc::new(RefCell::new(None::<JsValue>));
    super::helpers::listen(trigger.as_ref(), "click", move |_| {
        let shared = shared.clone();
        let held = held.clone();
        spawn_local(async move {
            let message = toggle_wake_lock(&held).await;
            shared.borrow_mut().active_probes.wake_lock_result = message;
            super::render::refresh(&shared);
        });
    })
}

async fn toggle_wake_lock(held: &Rc<RefCell<Option<JsValue>>>) -> String {
    let current = held.borrow_mut().take();
    if let Some(sentinel) = current {
        return match await_call0(&sentinel, "release").await {
            Ok(_) => "released".to_owned(),
            Err(error) => error_text(&error),
        };
    }
    let result = async {
        let navigator: JsValue = window()?.navigator().into();
        let wake_lock = property(&navigator, "wakeLock")
            .ok_or_else(|| JsValue::from_str("Wake Lock unavailable"))?;
        await_call1(&wake_lock, "request", &JsValue::from_str("screen")).await
    }
    .await;
    match result {
        Ok(sentinel) => {
            held.borrow_mut().replace(sentinel);
            "held".to_owned()
        }
        Err(error) => error_text(&error),
    }
}

fn register_service_worker(report: &SharedReport) {
    let shared = report.clone();
    spawn_local(async move {
        let result = async {
            let navigator: JsValue = window()?.navigator().into();
            let service_worker = property(&navigator, "serviceWorker")
                .ok_or_else(|| JsValue::from_str("Service Worker unavailable"))?;
            await_call1(&service_worker, "register", &JsValue::from_str("/sw.js")).await
        }
        .await;
        shared.borrow_mut().active_probes.service_worker_result =
            result.map_or_else(|error| error_text(&error), |_| "registered".to_owned());
        super::render::refresh(&shared);
    });
}

async fn await_call0(object: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    JsFuture::from(Promise::resolve(&call0(object, name)?)).await
}

async fn await_call1(object: &JsValue, name: &str, argument: &JsValue) -> Result<JsValue, JsValue> {
    JsFuture::from(Promise::resolve(&call1(object, name, argument)?)).await
}
