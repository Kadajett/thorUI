use super::SharedReport;
use super::helpers::{button, document, iso_now, set_text, window};
use crate::report::FrameRun;
use crate::statistics::summarize;
use js_sys::{Function, Promise};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{HtmlButtonElement, HtmlSelectElement};

const SAMPLE_DURATION_MS: f64 = 5_000.0;

pub fn install(report: &SharedReport) -> Result<(), JsValue> {
    let trigger = button("measure-frames")?;
    let shared = report.clone();
    let control = trigger.clone();
    super::helpers::listen(trigger.as_ref(), "click", move |_| {
        let shared = shared.clone();
        let control = control.clone();
        control.set_disabled(true);
        set_text("metric-frame", "Measuring…");
        spawn_local(async move {
            let result = measure().await;
            finish(&shared, &control, result);
        });
    })
}

async fn measure() -> Result<(Vec<f64>, u16), JsValue> {
    let expected_hz = expected_refresh()?;
    let start = next_frame().await?;
    let mut previous = start;
    let mut intervals = Vec::with_capacity(720);
    while previous - start < SAMPLE_DURATION_MS {
        let current = next_frame().await?;
        intervals.push(current - previous);
        previous = current;
    }
    Ok((intervals, expected_hz))
}

fn finish(
    report: &SharedReport,
    control: &HtmlButtonElement,
    result: Result<(Vec<f64>, u16), JsValue>,
) {
    control.set_disabled(false);
    match result {
        Ok((samples, expected_hz)) => record(report, &samples, expected_hz),
        Err(error) => set_text("metric-frame", &format!("Failed: {error:?}")),
    }
    super::render::refresh(report);
}

fn record(report: &SharedReport, samples: &[f64], expected_hz: u16) {
    let budget_ms = 1_000.0 / f64::from(expected_hz);
    let mut target = report.borrow_mut();
    let visibility = document().map_or_else(
        |_| "Unknown".to_owned(),
        |doc| format!("{:?}", doc.visibility_state()),
    );
    let surface_role = target.surface.role.clone();
    target.frame_runs.push(FrameRun {
        started_at: iso_now(),
        surface_role,
        visibility,
        expected_hz,
        budget_ms,
        distribution: summarize(samples, budget_ms * 1.25),
    });
}

fn expected_refresh() -> Result<u16, JsValue> {
    let select: HtmlSelectElement = super::helpers::element("expected-refresh")?.dyn_into()?;
    select
        .value()
        .parse()
        .map_err(|error| JsValue::from_str(&format!("invalid refresh target: {error}")))
}

pub async fn next_frame() -> Result<f64, JsValue> {
    let promise = Promise::new(&mut |resolve: Function, reject: Function| {
        let resolve_copy = resolve.clone();
        let callback = Closure::once_into_js(move |timestamp: f64| {
            let _ = resolve_copy.call1(&JsValue::UNDEFINED, &JsValue::from_f64(timestamp));
        });
        if let Err(error) = window().and_then(|host| {
            host.request_animation_frame(callback.unchecked_ref())
                .map(|_| ())
        }) {
            let _ = reject.call1(&JsValue::UNDEFINED, &error);
        }
    });
    JsFuture::from(promise)
        .await?
        .as_f64()
        .ok_or_else(|| JsValue::from_str("frame timestamp is not a number"))
}
