use super::Runtime;
use crate::showcase::ShowcaseAction;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{BroadcastChannel, MessageEvent};

#[derive(Debug, Deserialize, Serialize)]
struct WireMessage {
    source: String,
    role: String,
    payload: WirePayload,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
enum WirePayload {
    Hello,
    Ready,
    Action(ShowcaseAction),
}

pub struct ShowcaseLink {
    channel: BroadcastChannel,
    source: String,
    role: String,
}

impl ShowcaseLink {
    pub fn new(session: &str, role: &str) -> Result<Self, JsValue> {
        Ok(Self {
            channel: BroadcastChannel::new(&format!("thorui-showcase-{session}"))?,
            source: super::super::helpers::capture_id(),
            role: role.to_owned(),
        })
    }

    pub fn announce(&self) {
        self.send(WirePayload::Hello);
    }

    pub fn send_action(&self, action: &ShowcaseAction) {
        self.send(WirePayload::Action(*action));
    }

    fn ready(&self) {
        self.send(WirePayload::Ready);
    }

    fn send(&self, payload: WirePayload) {
        let message = WireMessage {
            source: self.source.clone(),
            role: self.role.clone(),
            payload,
        };
        if let Ok(json) = serde_json::to_string(&message) {
            let _ = self.channel.post_message(&JsValue::from_str(&json));
        }
    }
}

pub fn install(runtime: &Rc<RefCell<Runtime>>) {
    let target = runtime.clone();
    let channel = runtime.borrow().link.channel.clone();
    let callback = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        let Some(json) = event.data().as_string() else {
            return;
        };
        let Ok(message) = serde_json::from_str::<WireMessage>(&json) else {
            return;
        };
        receive(&target, &message);
    });
    channel.set_onmessage(Some(callback.as_ref().unchecked_ref()));
    callback.forget();
}

fn receive(runtime: &Rc<RefCell<Runtime>>, message: &WireMessage) {
    let mut target = runtime.borrow_mut();
    if message.source == target.link.source || message.role == target.role {
        return;
    }
    match &message.payload {
        WirePayload::Hello => {
            target.apply_remote(ShowcaseAction::SetPeer { connected: true });
            target.link.ready();
        }
        WirePayload::Ready => {
            target.apply_remote(ShowcaseAction::SetPeer { connected: true });
        }
        WirePayload::Action(action) => target.apply_remote(*action),
    }
}
