use super::SharedReport;
use super::helpers::{
    bool_property, display_mode, iso_now, number_property, property, string_property, supports,
    window,
};
use crate::report::{BuildIdentity, Insets, SupportMatrix};
use js_sys::global;
use wasm_bindgen::JsValue;

pub fn observe(report: &SharedReport) -> Result<(), JsValue> {
    let host = window()?;
    let navigator: JsValue = host.navigator().into();
    let screen: JsValue = host.screen()?.into();
    let host_value: JsValue = host.into();
    let root = global();
    let display_mode = display_mode();
    let mut target = report.borrow_mut();
    target.captured_at = iso_now();
    target.build = BuildIdentity {
        revision: option_env!("THORUI_BUILD_REVISION")
            .unwrap_or("development")
            .to_owned(),
        channel: option_env!("THORUI_CHANNEL").unwrap_or("local").to_owned(),
    };
    observe_surface(
        &mut target.surface,
        &host_value,
        &navigator,
        &screen,
        display_mode,
    );
    target.support = observe_support(&root, &host_value, &navigator);
    Ok(())
}

fn observe_surface(
    surface: &mut crate::report::SurfaceObservation,
    host: &JsValue,
    navigator: &JsValue,
    screen: &JsValue,
    display_mode: String,
) {
    surface.user_agent = string_property(navigator, "userAgent");
    surface.platform = string_property(navigator, "platform");
    surface.language = string_property(navigator, "language");
    surface.viewport_width_css = number_property(host, "innerWidth");
    surface.viewport_height_css = number_property(host, "innerHeight");
    surface.viewport_width_physical =
        surface.viewport_width_css * number_property(host, "devicePixelRatio");
    surface.viewport_height_physical =
        surface.viewport_height_css * number_property(host, "devicePixelRatio");
    surface.screen_width_css = number_property(screen, "width");
    surface.screen_height_css = number_property(screen, "height");
    surface.screen_width_physical =
        surface.screen_width_css * number_property(host, "devicePixelRatio");
    surface.screen_height_physical =
        surface.screen_height_css * number_property(host, "devicePixelRatio");
    surface.available_width_css = number_property(screen, "availWidth");
    surface.available_height_css = number_property(screen, "availHeight");
    surface.device_pixel_ratio = number_property(host, "devicePixelRatio");
    surface.color_depth = number_property(screen, "colorDepth");
    surface.pixel_depth = number_property(screen, "pixelDepth");
    let orientation = property(screen, "orientation").unwrap_or(JsValue::UNDEFINED);
    surface.orientation_type = string_property(&orientation, "type");
    surface.orientation_angle = number_property(&orientation, "angle");
    surface.hardware_concurrency = number_property(navigator, "hardwareConcurrency");
    surface.max_touch_points = number_property(navigator, "maxTouchPoints");
    surface.secure_context = bool_property(host, "isSecureContext");
    surface.cross_origin_isolated = bool_property(host, "crossOriginIsolated");
    surface.display_mode = display_mode;
    surface.safe_area = observe_safe_area().unwrap_or_default();
}

fn observe_safe_area() -> Result<Insets, JsValue> {
    let document = super::helpers::document()?;
    let probe = document.create_element("div")?;
    probe.set_attribute(
        "style",
        "position:fixed;visibility:hidden;padding:env(safe-area-inset-top) env(safe-area-inset-right) env(safe-area-inset-bottom) env(safe-area-inset-left)",
    )?;
    let body = document
        .body()
        .ok_or_else(|| JsValue::from_str("document body missing"))?;
    body.append_child(&probe)?;
    let style = window()?
        .get_computed_style(&probe)?
        .ok_or_else(|| JsValue::from_str("computed style missing"))?;
    let insets = Insets {
        top_css: css_pixels(&style.get_property_value("padding-top")?),
        right_css: css_pixels(&style.get_property_value("padding-right")?),
        bottom_css: css_pixels(&style.get_property_value("padding-bottom")?),
        left_css: css_pixels(&style.get_property_value("padding-left")?),
    };
    body.remove_child(&probe)?;
    Ok(insets)
}

fn css_pixels(value: &str) -> f64 {
    value.trim_end_matches("px").parse().unwrap_or_default()
}

fn observe_support(root: &JsValue, host: &JsValue, navigator: &JsValue) -> SupportMatrix {
    let document = property(host, "document").unwrap_or(JsValue::UNDEFINED);
    let storage = property(navigator, "storage").unwrap_or(JsValue::UNDEFINED);
    SupportMatrix {
        broadcast_channel: supports(root, "BroadcastChannel"),
        gamepad: supports(navigator, "getGamepads"),
        pointer_events: supports(root, "PointerEvent"),
        fullscreen: supports(&document, "fullscreenEnabled"),
        window_management: supports(host, "getScreenDetails"),
        presentation: supports(root, "PresentationRequest"),
        wake_lock: supports(navigator, "wakeLock"),
        service_worker: supports(navigator, "serviceWorker"),
        storage_manager: supports(&storage, "estimate"),
        webgl2: supports_webgl2(&document),
        webgpu: supports(navigator, "gpu"),
        audio_context: supports(root, "AudioContext") || supports(root, "webkitAudioContext"),
        offscreen_canvas: supports(root, "OffscreenCanvas"),
        installed_display_mode: display_mode() != "browser",
    }
}

fn supports_webgl2(document: &JsValue) -> bool {
    let Some(canvas) =
        super::helpers::call1(document, "createElement", &JsValue::from_str("canvas")).ok()
    else {
        return false;
    };
    super::helpers::call1(&canvas, "getContext", &JsValue::from_str("webgl2"))
        .is_ok_and(|context| !context.is_null())
}
