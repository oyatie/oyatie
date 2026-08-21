//! In-memory adapters for the two ports: a DSR repository and a
//! microservice/handler registry.
//!
//! These are the reference implementations the cascade is tested against,
//! and the substitute for the Postgres + Workflow adapters that IP-009
//! names (see the crate-level "Gaps"). They use `std::sync::Mutex` because
//! the ports take `&self` — the durable adapter will do the same over a
//! connection pool.
//!
//! The store is keyed on [`DsrRequestKey`], not on a bare request id: two
//! tenants numbering their requests independently WILL collide on the id
//! alone, and a collision there would let one certificate discharge two
//! tenants' erasure obligations.

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::kernel::{
    DsrKernelError, DsrRequest, DsrRequestKey, DsrRequestRepository, ErasureHandler,
    ErasureReceipt, HandlerFailure, MicroserviceRegistry, ProofOfErasure,
};

/// Everything stored about one request.
#[derive(Clone, Debug, Default)]
struct RequestRecord {
    receipts: BTreeMap<String, ErasureReceipt>,
    proof: Option<ProofOfErasure>,
}

/// A volatile [`DsrRequestRepository`], durable only for the process.
#[derive(Debug, Default)]
pub struct InMemoryDsrRepository {
    requests: Mutex<BTreeMap<DsrRequestKey, RequestRecord>>,
}

impl InMemoryDsrRepository {
    /// An empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many receipts are recorded for a request (0 if unknown).
    ///
    /// # Errors
    /// [`DsrKernelError::RepositoryUnavailable`] if the lock is poisoned.
    pub fn receipt_count(&self, key: &DsrRequestKey) -> Result<usize, DsrKernelError> {
        let guard = self
            .requests
            .lock()
            .map_err(|_| DsrKernelError::RepositoryUnavailable)?;
        Ok(guard.get(key).map_or(0, |record| record.receipts.len()))
    }

    /// How many distinct (tenant, request) records exist.
    ///
    /// # Errors
    /// [`DsrKernelError::RepositoryUnavailable`] if the lock is poisoned.
    pub fn open_request_count(&self) -> Result<usize, DsrKernelError> {
        let guard = self
            .requests
            .lock()
            .map_err(|_| DsrKernelError::RepositoryUnavailable)?;
        Ok(guard.len())
    }
}

impl DsrRequestRepository for InMemoryDsrRepository {
    fn open(&self, request: &DsrRequest) -> Result<(), DsrKernelError> {
        request.validate_for_erasure()?;
        let mut guard = self
            .requests
            .lock()
            .map_err(|_| DsrKernelError::RepositoryUnavailable)?;
        // Re-opening is a no-op: a retried submission must not erase the
        // receipts the first submission already collected. A different
        // tenant presenting the same id lands on a DIFFERENT key.
        guard.entry(request.key()).or_default();
        Ok(())
    }

    fn append_receipt(&self, receipt: &ErasureReceipt) -> Result<(), DsrKernelError> {
        let mut guard = self
            .requests
            .lock()
            .map_err(|_| DsrKernelError::RepositoryUnavailable)?;
        let record = guard
            .get_mut(&receipt.key())
            .ok_or(DsrKernelError::UnknownRequest)?;
        if record.proof.is_some() {
            return Err(DsrKernelError::AlreadyFinalized);
        }
        if record.receipts.contains_key(&receipt.microservice) {
            return Err(DsrKernelError::DuplicateReceipt);
        }
        record
            .receipts
            .insert(receipt.microservice.clone(), receipt.clone());
        Ok(())
    }

    fn finalize(&self, proof: &ProofOfErasure) -> Result<(), DsrKernelError> {
        let mut guard = self
            .requests
            .lock()
            .map_err(|_| DsrKernelError::RepositoryUnavailable)?;
        let record = guard
            .get_mut(&proof.key())
            .ok_or(DsrKernelError::UnknownRequest)?;
        if record.proof.is_some() {
            return Err(DsrKernelError::AlreadyFinalized);
        }
        record.proof = Some(proof.clone());
        Ok(())
    }

    fn receipt(
        &self,
        key: &DsrRequestKey,
        microservice: &str,
    ) -> Result<Option<ErasureReceipt>, DsrKernelError> {
        let guard = self
            .requests
            .lock()
            .map_err(|_| DsrKernelError::RepositoryUnavailable)?;
        let record = guard.get(key).ok_or(DsrKernelError::UnknownRequest)?;
        Ok(record.receipts.get(microservice).cloned())
    }

    fn receipts(&self, key: &DsrRequestKey) -> Result<Vec<ErasureReceipt>, DsrKernelError> {
        let guard = self
            .requests
            .lock()
            .map_err(|_| DsrKernelError::RepositoryUnavailable)?;
        let record = guard.get(key).ok_or(DsrKernelError::UnknownRequest)?;
        Ok(record.receipts.values().cloned().collect())
    }

    fn proof(&self, key: &DsrRequestKey) -> Result<Option<ProofOfErasure>, DsrKernelError> {
        let guard = self
            .requests
            .lock()
            .map_err(|_| DsrKernelError::RepositoryUnavailable)?;
        let record = guard.get(key).ok_or(DsrKernelError::UnknownRequest)?;
        Ok(record.proof.clone())
    }
}

/// A handler that reports success with a digest derived from the request,
/// counting its invocations so tests can prove the cascade did not re-run
/// an erasure that already produced a receipt.
#[derive(Debug, Default)]
pub struct RecordingHandler {
    microservice: String,
    invocations: Mutex<usize>,
    failure: Option<String>,
}

impl RecordingHandler {
    /// A handler that succeeds for `microservice`.
    #[must_use]
    pub fn succeeding(microservice: &str) -> Self {
        Self {
            microservice: microservice.to_owned(),
            invocations: Mutex::new(0),
            failure: None,
        }
    }

    /// A handler that always refuses with `detail`.
    #[must_use]
    pub fn failing(microservice: &str, detail: &str) -> Self {
        Self {
            microservice: microservice.to_owned(),
            invocations: Mutex::new(0),
            failure: Some(detail.to_owned()),
        }
    }

    /// How many times this handler was invoked.
    ///
    /// # Errors
    /// [`DsrKernelError::RepositoryUnavailable`] if the lock is poisoned.
    pub fn invocations(&self) -> Result<usize, DsrKernelError> {
        self.invocations
            .lock()
            .map(|guard| *guard)
            .map_err(|_| DsrKernelError::RepositoryUnavailable)
    }
}

impl ErasureHandler for RecordingHandler {
    fn erase(&self, request: &DsrRequest) -> Result<[u8; 32], HandlerFailure> {
        if let Ok(mut guard) = self.invocations.lock() {
            *guard += 1;
        }
        if let Some(detail) = &self.failure {
            return Err(HandlerFailure::new(detail));
        }
        // Stand-in for a real evidence digest over the deleted keyspace.
        // The tenant and the subject are both bound in, so two tenants
        // sharing a request id do not produce the same digest.
        let mut hasher = crate::digest::Sha256::new();
        hasher.update(self.microservice.as_bytes());
        hasher.update(b"|");
        hasher.update(request.tenant_id.as_bytes());
        hasher.update(b"|");
        hasher.update(request.id.0.as_bytes());
        hasher.update(b"|");
        hasher.update(request.subject_id.as_bytes());
        Ok(hasher.finalize())
    }
}

/// A volatile [`MicroserviceRegistry`] holding boxed handlers.
#[derive(Default)]
pub struct InMemoryRegistry {
    handlers: BTreeMap<String, Box<dyn ErasureHandler + Send + Sync>>,
    /// Microservices listed as active but deliberately without a handler —
    /// the IP-009 "registered without a DSR handler" halt condition.
    handlerless: Vec<String>,
}

impl core::fmt::Debug for InMemoryRegistry {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("InMemoryRegistry")
            .field("handlers", &self.handlers.keys().collect::<Vec<_>>())
            .field("handlerless", &self.handlerless)
            .finish()
    }
}

impl InMemoryRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register `microservice` with an erasure handler.
    pub fn register(
        &mut self,
        microservice: &str,
        handler: Box<dyn ErasureHandler + Send + Sync>,
    ) -> &mut Self {
        self.handlers.insert(microservice.to_owned(), handler);
        self
    }

    /// Register `microservice` as active but WITHOUT a handler.
    pub fn register_without_handler(&mut self, microservice: &str) -> &mut Self {
        self.handlerless.push(microservice.to_owned());
        self
    }

    /// Decommission `microservice`: it stops being listed as active, which
    /// is the ordinary mid-window event a shrinking registry represents.
    pub fn decommission(&mut self, microservice: &str) -> &mut Self {
        self.handlers.remove(microservice);
        self.handlerless.retain(|name| name != microservice);
        self
    }
}

impl MicroserviceRegistry for InMemoryRegistry {
    fn list_active(&self) -> Result<Vec<String>, DsrKernelError> {
        let mut names: Vec<String> = self.handlers.keys().cloned().collect();
        names.extend(self.handlerless.iter().cloned());
        Ok(names)
    }

    fn handler(&self, microservice: &str) -> Option<&dyn ErasureHandler> {
        self.handlers
            .get(microservice)
            .map(|boxed| boxed.as_ref() as &dyn ErasureHandler)
    }
}
