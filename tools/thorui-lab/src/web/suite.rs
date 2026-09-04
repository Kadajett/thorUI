use super::SharedReport;
use super::helpers::{button, set_text};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

pub fn install(report: &SharedReport) -> Result<(), JsValue> {
    let trigger = button("run-suite")?;
    let control = trigger.clone();
    let shared = report.clone();
    super::helpers::listen(trigger.as_ref(), "click", move |_| {
        control.set_disabled(true);
        super::input::sample_controllers(&shared);
        set_text(
            "suite-status",
            "1 of 4 · For 15 seconds: use every control and touch the pad.",
        );
        let control = control.clone();
        let shared = shared.clone();
        spawn_local(async move {
            let result = run(&shared).await;
            control.set_disabled(false);
            show_result(result);
        });
    })
}

async fn run(report: &SharedReport) -> Result<String, JsValue> {
    super::input::capture(report).await;
    super::render::refresh(report);
    set_text("suite-status", "2 of 4 · Checking browser capabilities…");
    super::active::run(report).await;
    set_text(
        "suite-status",
        "3 of 4 · Measuring this surface for 5 seconds…",
    );
    super::frames::run(report).await?;
    set_text("suite-status", "4 of 4 · Saving the report…");
    super::upload::submit(report)
        .await
        .map(|receipt| receipt.receipt_id)
}

fn show_result(result: Result<String, JsValue>) {
    match result {
        Ok(receipt) => set_text(
            "suite-status",
            &format!("Saved. Receipt {receipt}. You are done on this surface."),
        ),
        Err(error) => set_text(
            "suite-status",
            &format!("Could not save: {}", super::helpers::error_text(&error)),
        ),
    }
}
