use super::SharedReport;
use super::helpers::{error_text, window};
use serde::Deserialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, RequestInit, Response};

#[derive(Deserialize)]
pub struct Receipt {
    pub receipt_id: String,
}

pub async fn submit(report: &SharedReport) -> Result<Receipt, JsValue> {
    let body = serde_json::to_string(&*report.borrow())
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    let options = RequestInit::new();
    options.set_method("POST");
    options.set_headers(&headers);
    options.set_body(&JsValue::from_str(&body));
    let value = JsFuture::from(window()?.fetch_with_str_and_init("/api/reports", &options)).await?;
    let response: Response = value.dyn_into()?;
    let text = JsFuture::from(response.text()?)
        .await?
        .as_string()
        .unwrap_or_default();
    if !response.ok() {
        return Err(JsValue::from_str(&response_error(response.status(), &text)));
    }
    serde_json::from_str(&text).map_err(|error| JsValue::from_str(&error.to_string()))
}

fn response_error(status: u16, body: &str) -> String {
    let detail = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| value.get("error")?.as_str().map(str::to_owned))
        .unwrap_or_else(|| "server rejected the report".to_owned());
    error_text(&JsValue::from_str(&format!("HTTP {status}: {detail}")))
}
