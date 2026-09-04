use crate::statistics::Distribution;
use serde::{Deserialize, Serialize};

pub const REPORT_SCHEMA: u16 = 1;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CapabilityReport {
    pub schema_version: u16,
    pub capture_id: String,
    pub captured_at: String,
    pub build: BuildIdentity,
    pub surface: SurfaceObservation,
    pub support: SupportMatrix,
    pub active_probes: ActiveProbes,
    pub frame_runs: Vec<FrameRun>,
    pub controllers: Vec<ControllerObservation>,
    pub pointer_samples: Vec<PointerSample>,
    pub lifecycle: Vec<LifecycleEvent>,
    pub peer_link: PeerLink,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BuildIdentity {
    pub revision: String,
    pub channel: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SurfaceObservation {
    pub role: String,
    pub session_id: String,
    pub user_agent: String,
    pub platform: String,
    pub language: String,
    pub viewport_width_css: f64,
    pub viewport_height_css: f64,
    pub viewport_width_physical: f64,
    pub viewport_height_physical: f64,
    pub screen_width_css: f64,
    pub screen_height_css: f64,
    pub screen_width_physical: f64,
    pub screen_height_physical: f64,
    pub available_width_css: f64,
    pub available_height_css: f64,
    pub device_pixel_ratio: f64,
    pub color_depth: f64,
    pub pixel_depth: f64,
    pub orientation_type: String,
    pub orientation_angle: f64,
    pub hardware_concurrency: f64,
    pub max_touch_points: f64,
    pub secure_context: bool,
    pub cross_origin_isolated: bool,
    pub display_mode: String,
    pub safe_area: Insets,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Insets {
    pub top_css: f64,
    pub right_css: f64,
    pub bottom_css: f64,
    pub left_css: f64,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SupportMatrix {
    pub broadcast_channel: bool,
    pub gamepad: bool,
    pub pointer_events: bool,
    pub fullscreen: bool,
    pub window_management: bool,
    pub presentation: bool,
    pub wake_lock: bool,
    pub service_worker: bool,
    pub storage_manager: bool,
    pub webgl2: bool,
    pub webgpu: bool,
    pub audio_context: bool,
    pub offscreen_canvas: bool,
    pub installed_display_mode: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ActiveProbes {
    pub display_count: Option<usize>,
    pub display_details: String,
    pub storage_usage_bytes: Option<f64>,
    pub storage_quota_bytes: Option<f64>,
    pub webgpu_adapter: String,
    pub audio_result: String,
    pub fullscreen_result: String,
    pub wake_lock_result: String,
    pub service_worker_result: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FrameRun {
    pub started_at: String,
    pub surface_role: String,
    pub visibility: String,
    pub expected_hz: u16,
    pub budget_ms: f64,
    pub distribution: Distribution,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ControllerObservation {
    pub sampled_at_ms: f64,
    pub index: u32,
    pub id: String,
    pub mapping: String,
    pub connected: bool,
    pub buttons: Vec<ButtonObservation>,
    pub axes: Vec<f64>,
    pub vibration_supported: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ButtonObservation {
    pub index: usize,
    pub pressed: bool,
    pub touched: bool,
    pub value: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PointerSample {
    pub sampled_at_ms: f64,
    pub event: String,
    pub pointer_id: i32,
    pub pointer_type: String,
    pub primary: bool,
    pub x: i32,
    pub y: i32,
    pub pressure: f32,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LifecycleEvent {
    pub sampled_at_ms: f64,
    pub event: String,
    pub visibility: String,
    pub focused: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PeerLink {
    pub peer_id: String,
    pub peer_role: String,
    pub connected: bool,
    pub sent: u32,
    pub received: u32,
    pub lost_or_reordered: u32,
    pub round_trip_ms: Distribution,
}

impl CapabilityReport {
    pub fn new(capture_id: String, role: String, session_id: String) -> Self {
        Self {
            schema_version: REPORT_SCHEMA,
            capture_id,
            surface: SurfaceObservation {
                role,
                session_id,
                ..SurfaceObservation::default()
            },
            ..Self::default()
        }
    }
}
