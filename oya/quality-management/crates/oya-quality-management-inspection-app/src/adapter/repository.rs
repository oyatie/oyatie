use crate::error::{Result, ServiceError, ServiceErrorKind};
use crate::usecase::{CommandEnvelope, CommandReceipt, RepositoryPort};
use std::collections::HashSet;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct InMemoryRepositoryFixture {
    reservations: Mutex<HashSet<(String, String)>>,
    receipts: Mutex<Vec<CommandReceipt>>,
}

impl RepositoryPort for InMemoryRepositoryFixture {
    fn reserve_idempotency_key(&self, envelope: &CommandEnvelope) -> Result<()> {
        let key = (
            envelope.context.actor.tenant_id.as_str().to_owned(),
            envelope.context.actor.idempotency_key.as_str().to_owned(),
        );
        let mut reservations = self.reservations.lock().map_err(|_| {
            ServiceError::configuration("repository fixture reservation lock poisoned")
        })?;
        if reservations.insert(key) {
            Ok(())
        } else {
            Err(ServiceError::new(
                ServiceErrorKind::Conflict,
                "idempotency key already reserved for tenant",
            ))
        }
    }

    fn persist_command_receipt(&self, receipt: &CommandReceipt) -> Result<()> {
        let mut receipts = self
            .receipts
            .lock()
            .map_err(|_| ServiceError::configuration("repository fixture receipt lock poisoned"))?;
        receipts.push(receipt.clone());
        Ok(())
    }
}
