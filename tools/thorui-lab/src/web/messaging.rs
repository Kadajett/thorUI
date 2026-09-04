use super::SharedReport;
use super::frames::next_frame;
use super::helpers::{button, capture_id, now, set_text};
use crate::statistics::summarize;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::spawn_local;
use web_sys::{BroadcastChannel, MessageEvent};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PeerMessage {
    kind: String,
    peer_id: String,
    role: String,
    sequence: u32,
    sent_at_ms: f64,
}

#[derive(Default)]
struct LinkState {
    next_sequence: u32,
    last_received: Option<u32>,
    round_trips: Vec<f64>,
}

pub struct PeerHandle {
    channel: BroadcastChannel,
    peer_id: String,
    role: String,
    link: Rc<RefCell<LinkState>>,
}

pub fn install(report: &SharedReport) -> Result<PeerHandle, JsValue> {
    let session = report.borrow().surface.session_id.clone();
    let role = report.borrow().surface.role.clone();
    let handle = PeerHandle {
        channel: BroadcastChannel::new(&format!("thorui-lab-{session}"))?,
        peer_id: capture_id(),
        role,
        link: Rc::new(RefCell::new(LinkState::default())),
    };
    install_receiver(&handle, report);
    install_ping(&handle, report)?;
    Ok(handle)
}

pub fn announce(handle: &PeerHandle, report: &SharedReport) -> Result<(), JsValue> {
    send(handle, report, "hello", 0, now())
}

fn install_receiver(handle: &PeerHandle, report: &SharedReport) {
    let channel = handle.channel.clone();
    let outbound = handle.channel.clone();
    let local_id = handle.peer_id.clone();
    let local_role = handle.role.clone();
    let link = handle.link.clone();
    let shared = report.clone();
    let callback = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        let Some(json) = event.data().as_string() else {
            return;
        };
        let Ok(message) = serde_json::from_str::<PeerMessage>(&json) else {
            return;
        };
        if message.peer_id == local_id {
            return;
        }
        observe_message(&shared, &link, &message);
        if message.kind == "hello" {
            let _ = post(&outbound, &local_id, &local_role, "ack", 0, now());
        } else if message.kind == "ping" {
            let _ = post(
                &outbound,
                &local_id,
                &local_role,
                "pong",
                message.sequence,
                message.sent_at_ms,
            );
        }
        super::render::refresh(&shared);
    });
    channel.set_onmessage(Some(callback.as_ref().unchecked_ref()));
    callback.forget();
}

fn observe_message(report: &SharedReport, link: &Rc<RefCell<LinkState>>, message: &PeerMessage) {
    let mut target = report.borrow_mut();
    target.peer_link.peer_id.clone_from(&message.peer_id);
    target.peer_link.peer_role.clone_from(&message.role);
    target.peer_link.connected = true;
    target.peer_link.received += 1;
    if message.kind != "pong" {
        return;
    }
    let mut state = link.borrow_mut();
    if state
        .last_received
        .is_some_and(|last| message.sequence != last + 1)
    {
        target.peer_link.lost_or_reordered += 1;
    }
    state.last_received = Some(message.sequence);
    state
        .round_trips
        .push((now() - message.sent_at_ms).max(0.0));
    target.peer_link.round_trip_ms = summarize(&state.round_trips, 8.34);
}

fn install_ping(handle: &PeerHandle, report: &SharedReport) -> Result<(), JsValue> {
    let trigger = button("ping-peer")?;
    let channel = handle.channel.clone();
    let peer_id = handle.peer_id.clone();
    let role = handle.role.clone();
    let link = handle.link.clone();
    let shared = report.clone();
    super::helpers::listen(trigger.as_ref(), "click", move |_| {
        if !shared.borrow().peer_link.connected {
            set_text(
                "peer-summary",
                "No peer is connected. Open the companion surface first.",
            );
            return;
        }
        let channel = channel.clone();
        let peer_id = peer_id.clone();
        let role = role.clone();
        let link = link.clone();
        let shared = shared.clone();
        spawn_local(async move { ping_series(&channel, &peer_id, &role, &link, &shared).await });
    })
}

async fn ping_series(
    channel: &BroadcastChannel,
    peer_id: &str,
    role: &str,
    link: &Rc<RefCell<LinkState>>,
    report: &SharedReport,
) {
    for _ in 0..32 {
        let sequence = next_sequence(link);
        if post(channel, peer_id, role, "ping", sequence, now()).is_ok() {
            report.borrow_mut().peer_link.sent += 1;
        }
        if next_frame().await.is_err() {
            break;
        }
    }
    super::render::refresh(report);
}

fn next_sequence(link: &Rc<RefCell<LinkState>>) -> u32 {
    let mut target = link.borrow_mut();
    target.next_sequence += 1;
    target.next_sequence
}

fn send(
    handle: &PeerHandle,
    report: &SharedReport,
    kind: &str,
    sequence: u32,
    sent_at: f64,
) -> Result<(), JsValue> {
    post(
        &handle.channel,
        &handle.peer_id,
        &handle.role,
        kind,
        sequence,
        sent_at,
    )?;
    report.borrow_mut().peer_link.sent += 1;
    Ok(())
}

fn post(
    channel: &BroadcastChannel,
    peer_id: &str,
    role: &str,
    kind: &str,
    sequence: u32,
    sent_at_ms: f64,
) -> Result<(), JsValue> {
    let message = PeerMessage {
        kind: kind.to_owned(),
        peer_id: peer_id.to_owned(),
        role: role.to_owned(),
        sequence,
        sent_at_ms,
    };
    let json =
        serde_json::to_string(&message).map_err(|error| JsValue::from_str(&error.to_string()))?;
    channel.post_message(&JsValue::from_str(&json))
}
