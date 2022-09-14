use near_sdk::serde::Serialize;
use near_sdk::{env, AccountId};

#[derive(Serialize, Debug)]
#[serde(tag = "standard")]
#[must_use = "don't forget to `.emit()` this event"]
#[serde(rename_all = "snake_case")]
pub(crate) enum NearEvent<'a> {
    Nep245(Nep245Event<'a>),
}

impl<'a> NearEvent<'a> {
    fn to_json_string(&self) -> String {
        // Events cannot fail to serialize so fine to panic on error
        #[allow(clippy::redundant_closure)]
        serde_json::to_string(self).ok().unwrap_or_else(|| env::abort())
    }

    fn to_json_event_string(&self) -> String {
        format!("EVENT_JSON:{}", self.to_json_string())
    }

    /// Logs the event to the host. This is required to ensure that the event is triggered
    /// and to consume the event.
    pub(crate) fn emit(self) {
        env::log_str(&self.to_json_event_string());
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
enum Nep245EventKind<'a> {
    PTMint(&'a [PTMint<'a>]),
    PTBurn(&'a [PTBurn<'a>]),
}

fn new_245<'a>(version: &'static str, event_kind: Nep245EventKind<'a>) -> NearEvent<'a> {
    NearEvent::Nep245(Nep245Event { version, event_kind })
}

fn new_245_v1(event_kind: Nep245EventKind) -> NearEvent {
    new_245("1.0.0", event_kind)
}

#[must_use]
#[derive(Serialize, Debug, Clone)]
pub struct PTMint<'a> {
    pub owner_id: &'a AccountId,
    pub token_ids: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<&'a str>
}

impl PTMint<'_> {
    pub fn emit(self) {
        Self::emit_many(&[self])
    }

    pub fn emit_many(data: &[PTMint<'_>]) {
        new_245_v1(Nep245EventKind::PTMint(data)).emit()
    }
}

#[must_use]
#[derive(Serialize, Debug, Clone)]
pub struct PTBurn<'a> {
    pub owner_id: &'a AccountId,
    pub token_ids: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<&'a str>
}

impl PTBurn<'_> {
    pub fn emit(self) {
        Self::emit_many(&[self])
    }

    pub fn emit_many(data: &[PTBurn<'_>]) {
        new_245_v1(Nep245EventKind::PTBurn(data)).emit()
    }
}

#[derive(Serialize, Debug)]
pub struct Nep245Event<'a> {
    version:  &'static str,
    #[serde(flatten)]
    event_kind: Nep245EventKind<'a>
}