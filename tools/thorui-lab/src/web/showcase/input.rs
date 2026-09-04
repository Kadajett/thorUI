use super::Runtime;
use crate::showcase::{LumenColor, Point, ShowcaseAction};
use js_sys::Array;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, KeyboardEvent, PointerEvent};

const DEAD_ZONE: f64 = 0.16;

#[derive(Default)]
pub struct ControllerState {
    cycle_pressed: bool,
    clear_pressed: bool,
}

pub fn install(runtime: &Rc<RefCell<Runtime>>) -> Result<(), JsValue> {
    install_pointer(runtime)?;
    install_keyboard(runtime)?;
    install_buttons(runtime)
}

pub fn poll_controller(runtime: &mut Runtime) {
    let Some(gamepad) = first_gamepad() else {
        return;
    };
    let (horizontal, vertical) = direction(&gamepad);
    let paint = button_pressed(&gamepad, 0);
    if horizontal.abs() > DEAD_ZONE || vertical.abs() > DEAD_ZONE || paint {
        runtime.apply_local(ShowcaseAction::Move {
            horizontal,
            vertical,
            paint,
        });
        super::super::helpers::set_text("input-mode", "Controller");
    }
    let cycle = button_pressed(&gamepad, 2);
    if cycle && !runtime.controller.cycle_pressed {
        runtime.apply_local(ShowcaseAction::CycleColor);
    }
    runtime.controller.cycle_pressed = cycle;
    let clear = button_pressed(&gamepad, 1);
    if clear && !runtime.controller.clear_pressed {
        runtime.apply_local(ShowcaseAction::Clear);
    }
    runtime.controller.clear_pressed = clear;
}

fn install_pointer(runtime: &Rc<RefCell<Runtime>>) -> Result<(), JsValue> {
    let canvas: HtmlCanvasElement = super::super::helpers::element("lumen-canvas")?.dyn_into()?;
    for event_name in ["pointerdown", "pointermove"] {
        let target = runtime.clone();
        let surface = canvas.clone();
        super::super::helpers::listen(canvas.as_ref(), event_name, move |event| {
            let Some(pointer) = event.dyn_ref::<PointerEvent>() else {
                return;
            };
            if event_name == "pointermove" && pointer.buttons() == 0 {
                return;
            }
            if event_name == "pointerdown" {
                let _ = surface.set_pointer_capture(pointer.pointer_id());
            }
            let point = normalized_point(&surface, pointer);
            target.borrow_mut().apply_local(ShowcaseAction::Paint {
                point,
                strength: f64::from(pointer.pressure()).max(0.45),
            });
            super::super::helpers::set_text("input-mode", "Touch");
        })?;
    }
    Ok(())
}

fn install_keyboard(runtime: &Rc<RefCell<Runtime>>) -> Result<(), JsValue> {
    let target = runtime.clone();
    super::super::helpers::listen(
        super::super::helpers::window()?.as_ref(),
        "keydown",
        move |event| {
            let Some(keyboard) = event.dyn_ref::<KeyboardEvent>() else {
                return;
            };
            let action = keyboard_action(&keyboard.key());
            if let Some(action) = action {
                keyboard.prevent_default();
                target.borrow_mut().apply_local(action);
                super::super::helpers::set_text("input-mode", "Keyboard");
            }
        },
    )
}

fn install_buttons(runtime: &Rc<RefCell<Runtime>>) -> Result<(), JsValue> {
    for (id, color) in [
        ("color-mint", LumenColor::Mint),
        ("color-cyan", LumenColor::Cyan),
        ("color-violet", LumenColor::Violet),
        ("color-coral", LumenColor::Coral),
        ("color-gold", LumenColor::Gold),
    ] {
        install_action(runtime, id, ShowcaseAction::SelectColor { color })?;
    }
    install_action(runtime, "clear-field", ShowcaseAction::Clear)
}

fn install_action(
    runtime: &Rc<RefCell<Runtime>>,
    id: &str,
    action: ShowcaseAction,
) -> Result<(), JsValue> {
    let target = runtime.clone();
    let trigger = super::super::helpers::element(id)?;
    super::super::helpers::listen(trigger.as_ref(), "click", move |_| {
        target.borrow_mut().apply_local(action);
        super::super::helpers::set_text("input-mode", "Touch");
    })
}

fn normalized_point(canvas: &HtmlCanvasElement, event: &PointerEvent) -> Point {
    let bounds = canvas.get_bounding_client_rect();
    Point {
        x: (f64::from(event.client_x()) - bounds.left()) / bounds.width().max(1.0),
        y: (f64::from(event.client_y()) - bounds.top()) / bounds.height().max(1.0),
    }
    .clamped()
}

fn keyboard_action(key: &str) -> Option<ShowcaseAction> {
    let (horizontal, vertical) = match key {
        "ArrowUp" => (0.0, -1.0),
        "ArrowDown" => (0.0, 1.0),
        "ArrowLeft" => (-1.0, 0.0),
        "ArrowRight" => (1.0, 0.0),
        " " => (0.0, 0.0),
        "x" | "X" => return Some(ShowcaseAction::CycleColor),
        "Backspace" => return Some(ShowcaseAction::Clear),
        _ => return None,
    };
    Some(ShowcaseAction::Move {
        horizontal,
        vertical,
        paint: key == " ",
    })
}

fn first_gamepad() -> Option<JsValue> {
    let navigator: JsValue = super::super::helpers::window().ok()?.navigator().into();
    let gamepads: Array = super::super::helpers::call0(&navigator, "getGamepads")
        .ok()?
        .dyn_into()
        .ok()?;
    gamepads.iter().find(|value| !value.is_null())
}

fn direction(gamepad: &JsValue) -> (f64, f64) {
    let axes = super::super::helpers::property(gamepad, "axes")
        .map_or_else(Array::new, |value| Array::from(&value));
    let mut horizontal = axes.get(0).as_f64().unwrap_or_default();
    let mut vertical = axes.get(1).as_f64().unwrap_or_default();
    horizontal += digital_axis(gamepad, 14, 15);
    vertical += digital_axis(gamepad, 12, 13);
    (dead_zone(horizontal), dead_zone(vertical))
}

fn digital_axis(gamepad: &JsValue, negative: u32, positive: u32) -> f64 {
    if button_pressed(gamepad, negative) {
        -1.0
    } else if button_pressed(gamepad, positive) {
        1.0
    } else {
        0.0
    }
}

fn button_pressed(gamepad: &JsValue, index: u32) -> bool {
    let buttons = super::super::helpers::property(gamepad, "buttons")
        .map_or_else(Array::new, |value| Array::from(&value));
    let button = buttons.get(index);
    super::super::helpers::bool_property(&button, "pressed")
        || super::super::helpers::number_property(&button, "value") > 0.5
}

fn dead_zone(value: f64) -> f64 {
    if value.abs() < DEAD_ZONE {
        0.0
    } else {
        value.clamp(-1.0, 1.0)
    }
}
