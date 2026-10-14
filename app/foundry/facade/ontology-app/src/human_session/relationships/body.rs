use foundry_ontology_query_domain::{
    EdgeConsent, KnowledgeGraphQueryError, KnowledgeGraphQueryRequest, TraversalDirection,
};
use serde::Deserialize;

#[derive(Clone, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Root {
    pub(super) entity_type: String, // data_class: INTERNAL_ONLY
    pub(super) object_ref: String,  // data_class: INTERNAL_ONLY
    pub(super) revision: u32,       // data_class: INTERNAL_ONLY
}
#[derive(Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Direction {
    Outbound,
    Inbound,
}
#[derive(Clone, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    pub(super) idempotency_key: String, // data_class: INTERNAL_ONLY
    pub(super) root: Root,
    pub(super) link_type: String, // data_class: INTERNAL_ONLY
    pub(super) direction: Direction,
    pub(super) observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}
impl Request {
    pub(super) fn parse(body: &str) -> Option<Self> {
        let shape: serde_json::Value = serde_json::from_str(body).ok()?;
        if !shape.is_object() || !shape.get("root")?.is_object() {
            return None;
        }
        let request: Self = serde_json::from_str(body).ok()?;
        let key = &request.idempotency_key;
        (!key.is_empty()
            && key.len() <= 128
            && key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
            && id(&request.root.entity_type, "ety_")
            && id(&request.root.object_ref, "ent_")
            && id(&request.link_type, "lty_")
            && request.observed_at_epoch_seconds <= 9_007_199_254_740_991)
            .then_some(request)
    }
    pub(super) fn query(
        &self,
        tenant: &str,
    ) -> Result<KnowledgeGraphQueryRequest, KnowledgeGraphQueryError> {
        KnowledgeGraphQueryRequest::new(
            tenant,
            format!("kgq_{}", self.idempotency_key),
            &self.root.object_ref,
            vec![self.link_type.clone()],
            1,
            0,
            self.observed_at_epoch_seconds,
            EdgeConsent::Granted(vec![self.link_type.clone()]),
            match self.direction {
                Direction::Outbound => TraversalDirection::Outbound,
                Direction::Inbound => TraversalDirection::Inbound,
            },
        )
    }
}
fn id(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
        && value.len() > prefix.len()
        && value.len() <= 256
        && value.trim() == value
        && !value.chars().any(char::is_control)
}
