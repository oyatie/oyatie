//! M02-P04-IP-002 — SSE transport kernel.
//!
//! Event stream shape + subscriber list. Reuses canonical `AuditEvent`.

use std::collections::BTreeMap;

use intelligence_api_rest_kernel::{ResponseStatus, UseCaseRequest, UseCaseResponse};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SseEvent {
    pub id: String,         // data_class: INTERNAL_ONLY
    pub event_type: String, // data_class: INTERNAL_ONLY
    pub data: String,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SseSubscriberList {
    pub subscribers: Vec<String>, // data_class: INTERNAL_ONLY
}

impl SseSubscriberList {
    pub fn subscribe(&mut self, sub: String) {
        if !self.subscribers.contains(&sub) {
            self.subscribers.push(sub);
        }
    }
    pub fn unsubscribe(&mut self, sub: &str) {
        self.subscribers.retain(|s| s != sub);
    }
    pub fn len(&self) -> usize {
        self.subscribers.len()
    }
    pub fn is_empty(&self) -> bool {
        self.subscribers.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SseRequest {
    pub channel: String,                   // data_class: INTERNAL_ONLY
    pub use_case_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub payload: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

impl UseCaseRequest for SseRequest {
    fn use_case_id(&self) -> &str {
        &self.use_case_id
    }
    fn tenant_id(&self) -> &str {
        &self.tenant_id
    }
    fn payload(&self) -> &BTreeMap<String, String> {
        &self.payload
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SseResponse {
    pub use_case_id: String,            // data_class: INTERNAL_ONLY
    pub status: ResponseStatus,         // data_class: INTERNAL_ONLY
    pub stream_id: String,              // data_class: INTERNAL_ONLY
    pub body: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

impl UseCaseResponse for SseResponse {
    fn use_case_id(&self) -> &str {
        &self.use_case_id
    }
    fn status(&self) -> ResponseStatus {
        self.status
    }
    fn body(&self) -> &BTreeMap<String, String> {
        &self.body
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscriber_list_dedup() {
        let mut list = SseSubscriberList::default();
        list.subscribe("a".into());
        list.subscribe("a".into());
        list.subscribe("b".into());
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn subscriber_unsubscribe() {
        let mut list = SseSubscriberList::default();
        list.subscribe("a".into());
        list.subscribe("b".into());
        list.unsubscribe("a");
        assert_eq!(list.subscribers, vec!["b".to_string()]);
    }

    #[test]
    fn sse_event_shape() {
        let ev = SseEvent {
            id: "1".into(),
            event_type: "account-state-change".into(),
            data: "{}".into(),
        };
        assert_eq!(ev.event_type, "account-state-change");
    }
}
