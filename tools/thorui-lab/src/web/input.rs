use super::SharedReport;
use super::frames::next_frame;
use super::helpers::{button, call1, element, now, set_text, string_property, supports, window};
use crate::report::{ButtonObservation, ControllerObservation, PointerSample};
use js_sys::Array;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Gamepad, GamepadButton, PointerEvent};

const CAPTURE_DURATION_MS: f64 = 15_000.0;
const MAX_CONTROLLER_SNAPSHOTS: usize = 1_024;
const MAX_POINTER_SAMPLES: usize = 512;

pub fn install(report: &SharedReport) -> Result<(), JsValue> {
    install_capture(report)?;
    install_pointer_events(report)?;
    for event in ["gamepadconnected", "gamepaddisconnected"] {
        let shared = report.clone();
        super::helpers::listen(window()?.as_ref(), event, move |_| {
            sample_controllers(&shared);
            super::render::refresh(&shared);
        })?;
    }
    Ok(())
}

fn install_capture(report: &SharedReport) -> Result<(), JsValue> {
    let trigger = button("capture-input")?;
    let control = trigger.clone();
    let shared = report.clone();
    super::helpers::listen(trigger.as_ref(), "click", move |_| {
        control.set_disabled(true);
        set_text(
            "controller-summary",
            "Capturing for 15 seconds. Exercise every control.",
        );
        let control = control.clone();
        let shared = shared.clone();
        spawn_local(async move {
            capture(&shared).await;
            control.set_disabled(false);
            super::render::refresh(&shared);
        });
    })
}

pub async fn capture(report: &SharedReport) {
    let Ok(start) = next_frame().await else {
        return;
    };
    let mut current = start;
    while current - start < CAPTURE_DURATION_MS {
        sample_controllers(report);
        let Ok(next) = next_frame().await else {
            return;
        };
        current = next;
    }
}

pub(super) fn sample_controllers(report: &SharedReport) {
    let Some(gamepads) = gamepads() else {
        return;
    };
    for value in gamepads.iter().filter(|value| !value.is_null()) {
        let Ok(gamepad) = value.dyn_into::<Gamepad>() else {
            continue;
        };
        let observation = observe_gamepad(&gamepad);
        record_if_changed(report, observation);
    }
}

pub(super) fn gamepads() -> Option<Array> {
    let navigator: JsValue = window().ok()?.navigator().into();
    super::helpers::call0(&navigator, "getGamepads")
        .ok()?
        .dyn_into()
        .ok()
}

fn observe_gamepad(gamepad: &Gamepad) -> ControllerObservation {
    let buttons = gamepad
        .buttons()
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            value
                .dyn_into::<GamepadButton>()
                .ok()
                .map(|button| observe_button(index, &button))
        })
        .collect();
    let axes = gamepad
        .axes()
        .iter()
        .filter_map(|axis| axis.as_f64())
        .map(round_axis)
        .collect();
    let value: &JsValue = gamepad.as_ref();
    ControllerObservation {
        sampled_at_ms: now(),
        index: gamepad.index(),
        id: gamepad.id(),
        mapping: string_property(value, "mapping"),
        connected: gamepad.connected(),
        buttons,
        axes,
        vibration_supported: supports(value, "vibrationActuator")
            || supports(value, "hapticActuators"),
    }
}

fn observe_button(index: usize, button: &GamepadButton) -> ButtonObservation {
    ButtonObservation {
        index,
        pressed: button.pressed(),
        touched: button.touched(),
        value: round_axis(button.value()),
    }
}

fn record_if_changed(report: &SharedReport, observation: ControllerObservation) {
    let mut target = report.borrow_mut();
    let changed = target
        .controllers
        .last()
        .is_none_or(|previous| signature(previous) != signature(&observation));
    if changed {
        target.controllers.push(observation);
        if target.controllers.len() > MAX_CONTROLLER_SNAPSHOTS {
            target.controllers.remove(0);
        }
    }
}

fn signature(controller: &ControllerObservation) -> String {
    let buttons = controller
        .buttons
        .iter()
        .map(|button| format!("{}:{:.2}", button.pressed, button.value))
        .collect::<Vec<_>>()
        .join("|");
    format!("{}:{:?}:{buttons}", controller.connected, controller.axes)
}

fn install_pointer_events(report: &SharedReport) -> Result<(), JsValue> {
    let target = element("touch-target")?;
    for name in [
        "pointerdown",
        "pointermove",
        "pointerup",
        "pointercancel",
        "lostpointercapture",
    ] {
        let shared = report.clone();
        let event_name = name.to_owned();
        let touch_target = target.clone();
        super::helpers::listen(target.as_ref(), name, move |event| {
            let Some(pointer) = event.dyn_ref::<PointerEvent>() else {
                return;
            };
            if event_name == "pointerdown" {
                let _ = call1(
                    touch_target.as_ref(),
                    "setPointerCapture",
                    &JsValue::from(pointer.pointer_id()),
                );
            }
            touch_target.set_class_name(
                if event_name == "pointerup" || event_name == "pointercancel" {
                    "touch-target"
                } else {
                    "touch-target active"
                },
            );
            record_pointer(&shared, &event_name, pointer);
            super::render::refresh(&shared);
        })?;
    }
    Ok(())
}

fn record_pointer(report: &SharedReport, name: &str, event: &PointerEvent) {
    let sample = PointerSample {
        sampled_at_ms: now(),
        event: name.to_owned(),
        pointer_id: event.pointer_id(),
        pointer_type: event.pointer_type(),
        primary: event.is_primary(),
        x: event.client_x(),
        y: event.client_y(),
        pressure: event.pressure(),
        width: f64::from(event.width()),
        height: f64::from(event.height()),
    };
    let summary = format!(
        "{} · {} #{} · {},{} · pressure {:.2}",
        sample.event, sample.pointer_type, sample.pointer_id, sample.x, sample.y, sample.pressure
    );
    let mut target = report.borrow_mut();
    target.pointer_samples.push(sample);
    if target.pointer_samples.len() > MAX_POINTER_SAMPLES {
        target.pointer_samples.remove(0);
    }
    drop(target);
    set_text("pointer-summary", &summary);
}

fn round_axis(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}
