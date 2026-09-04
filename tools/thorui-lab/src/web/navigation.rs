use super::frames::next_frame;
use super::helpers::{bool_property, document, property, set_text, string_property};
use crate::focus::{Direction, Rect, choose};
use js_sys::Array;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, HtmlElement};

const AXIS_THRESHOLD: f64 = 0.55;
const INITIAL_REPEAT_MS: f64 = 360.0;
const REPEAT_MS: f64 = 120.0;

#[derive(Default)]
struct NavigationState {
    direction: Option<Direction>,
    confirm: bool,
    next_repeat_at: f64,
}

pub fn install() {
    spawn_local(async move {
        let mut state = NavigationState::default();
        while let Ok(timestamp) = next_frame().await {
            let Some(gamepad) = first_gamepad() else {
                continue;
            };
            update(&mut state, &gamepad, timestamp);
        }
    });
}

fn update(state: &mut NavigationState, gamepad: &JsValue, timestamp: f64) {
    let direction = direction(gamepad);
    if should_move(state, direction, timestamp) {
        move_focus(direction.unwrap_or(Direction::Down));
        mark_controller_mode(gamepad);
    }
    let confirm = button_pressed(gamepad, 0);
    if confirm && !state.confirm {
        activate_focus();
        mark_controller_mode(gamepad);
    }
    state.confirm = confirm;
}

fn should_move(state: &mut NavigationState, direction: Option<Direction>, timestamp: f64) -> bool {
    if direction.is_none() {
        state.direction = None;
        return false;
    }
    if state.direction != direction {
        state.direction = direction;
        state.next_repeat_at = timestamp + INITIAL_REPEAT_MS;
        return true;
    }
    if timestamp >= state.next_repeat_at {
        state.next_repeat_at = timestamp + REPEAT_MS;
        return true;
    }
    false
}

fn direction(gamepad: &JsValue) -> Option<Direction> {
    let axes = property(gamepad, "axes").map_or_else(Array::new, |value| Array::from(&value));
    let horizontal = axes.get(0).as_f64().unwrap_or_default();
    let vertical = axes.get(1).as_f64().unwrap_or_default();
    if button_pressed(gamepad, 12) || vertical < -AXIS_THRESHOLD {
        Some(Direction::Up)
    } else if button_pressed(gamepad, 13) || vertical > AXIS_THRESHOLD {
        Some(Direction::Down)
    } else if button_pressed(gamepad, 14) || horizontal < -AXIS_THRESHOLD {
        Some(Direction::Left)
    } else if button_pressed(gamepad, 15) || horizontal > AXIS_THRESHOLD {
        Some(Direction::Right)
    } else {
        None
    }
}

fn button_pressed(gamepad: &JsValue, index: u32) -> bool {
    let buttons = property(gamepad, "buttons").map_or_else(Array::new, |value| Array::from(&value));
    let button = buttons.get(index);
    bool_property(&button, "pressed") || super::helpers::number_property(&button, "value") > 0.5
}

fn first_gamepad() -> Option<JsValue> {
    let gamepads: Array = super::input::gamepads()?;
    gamepads.iter().find(|value| !value.is_null())
}

fn move_focus(direction: Direction) {
    let Ok(controls) = focusable_controls() else {
        return;
    };
    if controls.is_empty() {
        return;
    }
    let active = document()
        .ok()
        .and_then(|host| host.active_element())
        .filter(|element| is_control(element, &controls));
    let target = active
        .as_ref()
        .and_then(|element| spatial_target(element, &controls, direction))
        .unwrap_or_else(|| controls[0].clone());
    if let Some(element) = target.dyn_ref::<HtmlElement>() {
        let _ = element.focus();
        element.scroll_into_view();
    }
}

fn is_control(active: &Element, controls: &[Element]) -> bool {
    controls
        .iter()
        .any(|control| control.is_same_node(Some(active)))
}

fn focusable_controls() -> Result<Vec<Element>, wasm_bindgen::JsValue> {
    let nodes = document()?
        .query_selector_all("button:not(:disabled),select,textarea,summary,[tabindex='0']")?;
    Ok((0..nodes.length())
        .filter_map(|index| nodes.get(index))
        .filter_map(|node| node.dyn_into::<Element>().ok())
        .filter(|element| element.get_bounding_client_rect().width() > 0.0)
        .collect())
}

fn spatial_target(active: &Element, controls: &[Element], direction: Direction) -> Option<Element> {
    let current = rect(active);
    let candidates = controls.iter().map(rect).collect::<Vec<_>>();
    choose(current, &candidates, direction).map(|index| controls[index].clone())
}

fn rect(element: &Element) -> Rect {
    let bounds = element.get_bounding_client_rect();
    Rect {
        left: bounds.left(),
        top: bounds.top(),
        width: bounds.width(),
        height: bounds.height(),
    }
}

fn activate_focus() {
    let Some(active) = document().ok().and_then(|host| host.active_element()) else {
        return;
    };
    if let Some(element) = active.dyn_ref::<HtmlElement>() {
        element.click();
    }
}

fn mark_controller_mode(gamepad: &JsValue) {
    if let Ok(Some(body)) = document().map(|host| host.body()) {
        body.set_class_name("controller-mode");
    }
    set_text(
        "controller-nav-status",
        &format!("Controller active · {}", string_property(gamepad, "id")),
    );
}
