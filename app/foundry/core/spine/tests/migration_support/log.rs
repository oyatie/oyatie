//! An honest in-memory log: dense per-tenant ordinals, byte-equality
//! dedup to the ORIGINAL receipt, loud divergent-key conflicts.

use foundry_records_draft::{ActionEnvelope, Receipt, RecordsLog, RecordsLogError, SealedEnvelope};

#[derive(Default)]
pub struct MemoryLog {
    entries: Vec<SealedEnvelope>,
}

impl MemoryLog {
    pub fn seed(&mut self, sealed: SealedEnvelope) {
        self.entries.push(sealed);
    }
}

impl RecordsLog for MemoryLog {
    fn append(&mut self, envelope: ActionEnvelope) -> Result<Receipt, RecordsLogError> {
        if let Some(existing) = self.entries.iter().find(|sealed| {
            sealed.envelope.tenant_id == envelope.tenant_id
                && sealed.envelope.idempotency_key == envelope.idempotency_key
        }) {
            if existing.envelope == envelope {
                return Ok(Receipt {
                    deduplicated: true,
                    ..existing.receipt.clone()
                });
            }
            return Err(RecordsLogError::IdempotencyConflict {
                tenant_id: envelope.tenant_id,
                idempotency_key: envelope.idempotency_key,
            });
        }
        let ordinal = self
            .entries
            .iter()
            .filter(|sealed| sealed.envelope.tenant_id == envelope.tenant_id)
            .count() as u64
            + 1;
        let object_sequence = self
            .entries
            .iter()
            .filter(|sealed| {
                sealed.envelope.tenant_id == envelope.tenant_id
                    && sealed.envelope.object_ref == envelope.object_ref
            })
            .count() as u64
            + 1;
        let receipt = Receipt {
            ordinal,
            object_sequence,
            deduplicated: false,
        };
        self.entries.push(SealedEnvelope {
            envelope,
            receipt: receipt.clone(),
        });
        Ok(receipt)
    }

    fn replay(
        &self,
        tenant_id: &str,
        from_ordinal: u64,
    ) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        Ok(self
            .entries
            .iter()
            .filter(|sealed| {
                sealed.envelope.tenant_id == tenant_id && sealed.receipt.ordinal >= from_ordinal
            })
            .cloned()
            .collect())
    }

    fn head(&self, tenant_id: &str) -> Result<u64, RecordsLogError> {
        Ok(self
            .entries
            .iter()
            .filter(|sealed| sealed.envelope.tenant_id == tenant_id)
            .count() as u64)
    }
}
