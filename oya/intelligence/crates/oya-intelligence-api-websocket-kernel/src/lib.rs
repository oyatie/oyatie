//! M02-P04-IP-002 — WebSocket transport kernel.
//!
//! Subscription shape + use-case projection.

use std::collections::BTreeMap;

use intelligence_api_rest_kernel::{ResponseStatus, UseCaseRequest, UseCaseResponse};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WsSubscription {
    pub topic: String,         // data_class: INTERNAL_ONLY
    pub subscriber_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WsSubscriptionTable {
    pub entries: Vec<WsSubscription>, // data_class: INTERNAL_ONLY
}

impl WsSubscriptionTable {
    pub fn subscribe(&mut self, sub: WsSubscription) {
        if !self.entries.contains(&sub) {
            self.entries.push(sub);
        }
    }
    pub fn unsubscribe(&mut self, topic: &str, subscriber_id: &str) {
        self.entries
            .retain(|e| !(e.topic == topic && e.subscriber_id == subscriber_id));
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WsRequest {
    pub topic: String,                     // data_class: INTERNAL_ONLY
    pub use_case_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub payload: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

impl UseCaseRequest for WsRequest {
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
pub struct WsResponse {
    pub use_case_id: String,            // data_class: INTERNAL_ONLY
    pub status: ResponseStatus,         // data_class: INTERNAL_ONLY
    pub frame_id: String,               // data_class: INTERNAL_ONLY
    pub body: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

impl UseCaseResponse for WsResponse {
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
    fn subscription_table_dedup() {
        let mut t = WsSubscriptionTable::default();
        let s = WsSubscription {
            topic: "x".into(),
            subscriber_id: "a".into(),
        };
        t.subscribe(s.clone());
        t.subscribe(s);
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn subscription_table_unsubscribe() {
        let mut t = WsSubscriptionTable::default();
        t.subscribe(WsSubscription {
            topic: "x".into(),
            subscriber_id: "a".into(),
        });
        t.subscribe(WsSubscription {
            topic: "x".into(),
            subscriber_id: "b".into(),
        });
        t.unsubscribe("x", "a");
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn ws_request_projects_use_case_request() {
        let req = WsRequest {
            topic: "foundry/events".into(),
            use_case_id: "foundry.account.view".into(),
            tenant_id: "tenant-alpha".into(),
            payload: BTreeMap::new(),
        };
        assert_eq!(req.use_case_id(), "foundry.account.view");
    }
}
