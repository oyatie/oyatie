//! Messaging domain kernel: idempotent outbox records for downstream broker publication.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod cloud_event;

pub use cloud_event::{CloudEvent, CloudEventError};

use std::collections::BTreeMap;

use oya_data_boundary_kernel::{Classified, DataClass};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxRecord {
    pub sequence: u64,                       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub topic: Classified<String>,           // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub payload_ref: Classified<String>,     // data_class: INTERNAL_ONLY
    pub published: bool,                     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Topic {
    axis: Classified<String>,        // data_class: INTERNAL_ONLY
    name: Classified<String>,        // data_class: INTERNAL_ONLY
    description: Classified<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventingError {
    EmptyTopic,
    EmptyTopicAxis,
    EmptyTopicDescription,
    InvalidTopicName,
    DuplicateTopic,
    TopicNotFound,
    EmptyIdempotencyKey,
    EmptyPayloadRef,
    IdempotencyReplayMismatch,
    OutboxRecordNotFound,
    InvalidOutboxHistory,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TopicRegistry {
    by_name: BTreeMap<String, Topic>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Outbox {
    records: Vec<OutboxRecord>, // data_class: INTERNAL_ONLY
    by_idempotency: BTreeMap<(String, String, String), usize>, // data_class: INTERNAL_ONLY
}

impl Topic {
    pub fn new(
        axis: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Self, EventingError> {
        let axis = axis.into();
        let name = name.into();
        let description = description.into();
        let axis_trimmed = axis.trim();
        let name_trimmed = name.trim();
        let description_trimmed = description.trim();

        if axis_trimmed.is_empty() {
            return Err(EventingError::EmptyTopicAxis);
        }
        if name_trimmed.is_empty() {
            return Err(EventingError::EmptyTopic);
        }
        if description_trimmed.is_empty() {
            return Err(EventingError::EmptyTopicDescription);
        }

        let expected_prefix = format!("oya.{axis_trimmed}.");
        if name_trimmed != name || !name_trimmed.starts_with(&expected_prefix) {
            return Err(EventingError::InvalidTopicName);
        }

        Ok(Self {
            axis: Classified::new(axis_trimmed.to_string(), DataClass::InternalOnly),
            name: Classified::new(name_trimmed.to_string(), DataClass::InternalOnly),
            description: Classified::new(description_trimmed.to_string(), DataClass::InternalOnly),
        })
    }

    pub fn axis(&self) -> &str {
        &self.axis.value
    }

    pub fn name(&self) -> &str {
        &self.name.value
    }

    pub fn description(&self) -> &str {
        &self.description.value
    }

    fn validate(&self) -> Result<(), EventingError> {
        let axis = self.axis();
        let name = self.name();
        let description = self.description();
        let axis_trimmed = axis.trim();
        let name_trimmed = name.trim();
        let description_trimmed = description.trim();

        if axis_trimmed.is_empty() {
            return Err(EventingError::EmptyTopicAxis);
        }
        if name_trimmed.is_empty() {
            return Err(EventingError::EmptyTopic);
        }
        if description_trimmed.is_empty() {
            return Err(EventingError::EmptyTopicDescription);
        }

        let expected_prefix = format!("oya.{axis_trimmed}.");
        if axis_trimmed != axis
            || name_trimmed != name
            || !name_trimmed.starts_with(&expected_prefix)
        {
            return Err(EventingError::InvalidTopicName);
        }
        Ok(())
    }
}

impl TopicRegistry {
    pub fn register(&mut self, topic: Topic) -> Result<Topic, EventingError> {
        topic.validate()?;
        let name = topic.name().to_string();
        if self.by_name.contains_key(&name) {
            return Err(EventingError::DuplicateTopic);
        }
        self.by_name.insert(name, topic.clone());
        Ok(topic)
    }

    pub fn get(&self, name: &str) -> Option<&Topic> {
        self.by_name.get(name)
    }

    pub fn require(&self, name: &str) -> Result<&Topic, EventingError> {
        self.get(name).ok_or(EventingError::TopicNotFound)
    }

    pub fn topics(&self) -> Vec<&Topic> {
        self.by_name.values().collect()
    }
}

impl Outbox {
    pub fn from_records(records: Vec<OutboxRecord>) -> Result<Self, EventingError> {
        let mut outbox = Self::default();
        for record in records {
            if record.sequence != outbox.records.len() as u64
                || record.topic.value.trim().is_empty()
                || record.idempotency_key.value.trim().is_empty()
                || record.payload_ref.value.trim().is_empty()
            {
                return Err(EventingError::InvalidOutboxHistory);
            }
            let key = (
                record.tenant_id.clone(),
                record.topic.value.clone(),
                record.idempotency_key.value.clone(),
            );
            if outbox.by_idempotency.contains_key(&key) {
                return Err(EventingError::InvalidOutboxHistory);
            }
            outbox.by_idempotency.insert(key, outbox.records.len());
            outbox.records.push(record);
        }
        Ok(outbox)
    }

    pub fn publish(
        &mut self,
        tenant_id: String,
        topic: String,
        idempotency_key: String,
        payload_ref: String,
    ) -> Result<OutboxRecord, EventingError> {
        if topic.trim().is_empty() {
            return Err(EventingError::EmptyTopic);
        }
        if idempotency_key.trim().is_empty() {
            return Err(EventingError::EmptyIdempotencyKey);
        }
        if payload_ref.trim().is_empty() {
            return Err(EventingError::EmptyPayloadRef);
        }
        let key = (tenant_id.clone(), topic.clone(), idempotency_key.clone());
        if let Some(index) = self.by_idempotency.get(&key) {
            if self.records[*index].payload_ref.value != payload_ref {
                return Err(EventingError::IdempotencyReplayMismatch);
            }
            return Ok(self.records[*index].clone());
        }
        let record = OutboxRecord {
            sequence: self.records.len() as u64,
            tenant_id,
            topic: Classified::new(topic, DataClass::InternalOnly),
            idempotency_key: Classified::new(idempotency_key, DataClass::InternalOnly),
            payload_ref: Classified::new(payload_ref, DataClass::InternalOnly),
            published: false,
        };
        self.by_idempotency.insert(key, self.records.len());
        self.records.push(record.clone());
        Ok(record)
    }

    pub fn records(&self) -> &[OutboxRecord] {
        &self.records
    }

    pub fn mark_published(
        &mut self,
        tenant_id: &str,
        sequence: u64,
    ) -> Result<OutboxRecord, EventingError> {
        let index = usize::try_from(sequence).map_err(|_| EventingError::OutboxRecordNotFound)?;
        let record = self
            .records
            .get_mut(index)
            .filter(|record| record.sequence == sequence && record.tenant_id == tenant_id)
            .ok_or(EventingError::OutboxRecordNotFound)?;
        record.published = true;
        Ok(record.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{Classified, DataClass, EventingError, Outbox, Topic, TopicRegistry};

    #[test]
    fn topic_registry_enforces_axis_prefixed_names() {
        let topic = Topic::new(
            "foundation",
            "oya.foundation.eventing",
            "Foundation eventing backbone topic",
        )
        .expect("axis-prefixed topic is valid");

        let mut registry = TopicRegistry::default();
        registry
            .register(topic.clone())
            .expect("first registration succeeds");

        assert_eq!(registry.require("oya.foundation.eventing"), Ok(&topic));
        assert_eq!(registry.topics(), vec![&topic]);
        assert_eq!(registry.register(topic), Err(EventingError::DuplicateTopic));
        assert_eq!(
            Topic::new("foundation", "oya.cloud.eventing", "wrong axis"),
            Err(EventingError::InvalidTopicName)
        );
        assert_eq!(
            Topic::new("", "oya.foundation.eventing", "missing axis"),
            Err(EventingError::EmptyTopicAxis)
        );
    }

    #[test]
    fn topic_registry_revalidates_topic_invariants_at_registration() {
        let invalid = Topic {
            axis: Classified::new("foundation".to_string(), DataClass::InternalOnly),
            name: Classified::new("oya.cloud.eventing".to_string(), DataClass::InternalOnly),
            description: Classified::new(
                "Foundation eventing backbone topic".to_string(),
                DataClass::InternalOnly,
            ),
        };

        let mut registry = TopicRegistry::default();
        assert_eq!(
            registry.register(invalid),
            Err(EventingError::InvalidTopicName)
        );
        assert!(registry.topics().is_empty());
    }

    #[test]
    fn outbox_publish_remains_exactly_once_per_tenant_topic_and_key() {
        let mut outbox = Outbox::default();
        let first = outbox
            .publish(
                "tenant-a".to_string(),
                "oya.foundation.eventing".to_string(),
                "idem-1".to_string(),
                "payloads/1".to_string(),
            )
            .expect("first publish succeeds");
        let replay = outbox
            .publish(
                "tenant-a".to_string(),
                "oya.foundation.eventing".to_string(),
                "idem-1".to_string(),
                "payloads/1".to_string(),
            )
            .expect("idempotent replay succeeds");

        assert_eq!(first, replay);
        assert_eq!(outbox.records().len(), 1);

        let published = outbox
            .mark_published("tenant-a", first.sequence)
            .expect("record can be marked published");
        assert!(published.published);
        assert!(outbox.records()[0].published);
    }

    #[test]
    fn outbox_rejects_same_idempotency_key_with_different_payload_ref() {
        let mut outbox = Outbox::default();
        outbox
            .publish(
                "tenant-a".to_string(),
                "oya.foundation.eventing".to_string(),
                "idem-1".to_string(),
                "payloads/1".to_string(),
            )
            .expect("first publish succeeds");

        assert_eq!(
            outbox.publish(
                "tenant-a".to_string(),
                "oya.foundation.eventing".to_string(),
                "idem-1".to_string(),
                "payloads/2".to_string(),
            ),
            Err(EventingError::IdempotencyReplayMismatch)
        );
        assert_eq!(outbox.records().len(), 1);
    }
}
