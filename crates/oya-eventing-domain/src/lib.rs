//! Eventing kernel: idempotent outbox records for downstream broker publication.

use std::collections::BTreeMap;

use oya_platform_data_boundary_kernel::{Classified, DataClass};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxRecord {
    pub sequence: u64,
    pub tenant_id: String,
    pub topic: Classified<String>,
    pub idempotency_key: Classified<String>,
    pub payload_ref: Classified<String>,
    pub published: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventingError {
    EmptyTopic,
    EmptyIdempotencyKey,
    EmptyPayloadRef,
    OutboxRecordNotFound,
    InvalidOutboxHistory,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Outbox {
    records: Vec<OutboxRecord>,
    by_idempotency: BTreeMap<(String, String, String), usize>,
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
